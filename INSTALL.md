Installation Instructions
=========================

## Install with TLS (default)

***These instructions modify nodes in your cluster. Only run on test clusters currently.***

The following instructions install the Trow registry on an existing Kubernetes
cluster, with a certificate signed by the Kubernetes CA. They have been
primarily tested with GKE and minikube from MacOS and Linux. 

We recommend spinning up a small GKE cluster for testing Trow to begin with.

### Pre-requisites

 - `kubectl` is installed and configured to point at the cluster you wish to install Trow on
 - You've cloned or downloaded this repo
 - Port 31000 can be reached on the worker nodes (you may need to edit the
   network policy on AWS or GKE)
 - If you're running on GKE or have RBAC configured you may need to expand your
   rights to be able to create the needed service-account (on GKE the user is probably your e-mail address):
```
$ kubectl create clusterrolebinding cluster-admin-binding --clusterrole=cluster-admin --user=<user>
clusterrolebinding.rbac.authorization.k8s.io "cluster-admin-binding" created
```

### Automatic installation

 - Just run `./install.sh` and follow the prompts. 

If you'd rather have more control over the process, follow the [manual
steps](./MANUAL_INSTALL.md).

### Test it out

Trow has configured the domain `trow.kube-public` to point to your kubernetes cluster. Try pushing an image:

```
$ docker pull nginx:alpine
alpine: Pulling from library/nginx
Digest: sha256:e0292d158b6b353fde34909243a4886977cb9d1abb8a8a5fef9e0ff7138dd3e2
Status: Image is up to date for nginx:alpine
$ docker tag nginx:alpine trow.kube-public:31000/test/nginx:alpine
$ docker push trow.kube-public:31000/test/nginx:alpine
The push refers to repository [trow.kube-public:31000/test/nginx]
979531bcfa2b: Pushed 
8d36c62f099e: Pushed 
4b735058ece4: Pushed 
503e53e365f3: Pushed 
alpine: digest: sha256:bfddb36c23addfd10db511d95b7508fa7b6b2aca09b313ff3ef73c3752d11a55 size: 11903
```

If the push seems to hang, check if port 31000 is blocked (common with cloud provider default network rules).

The Kubernetes cluster should now be able to pull and run the image:

```
$ kubectl run trow-test --image=trow.kube-public:31000/test/nginx:alpine
deployment.apps "trow-test" created
$ kubectl get deploy trow-test
NAME        DESIRED   CURRENT   UP-TO-DATE   AVAILABLE   AGE
trow-test   1         1         1            1           8s
```

_NOTE: Don't worry if you get a message about run being deprecated. The command still works and I'll update these instructions in the future_

If you have enabled validation of images, try running a Docker Hub image, which should be denied:

```
$ kubectl run proxy --image=docker.io/nginx
deployment.apps "proxy" created
$ kubectl get deployment proxy
NAME      DESIRED   CURRENT   UP-TO-DATE   AVAILABLE   AGE
proxy     1         0         0            0           13s
$ kubectl describe rs proxy
...
  Warning  FailedCreate  16s (x13 over 57s)  replicaset-controller  Error creating: admission webhook "validator.trow.io" denied the request: Remote image docker.io/nginx disallowed as not contained in this registry and not in allow list
```

If you want to allow images from the Docker Hub, take a look at the `--allow-docker-official` and `--allow-prefixes` arguments. This can be passed to Trow via the `trow.yaml` file.

### Enable Authentication

At this time the only authentication available is a simple username & password combination that can be set when starting Trow. To enable this, use the `-p` and `-u` arguments, which can be set in the appropriate section of the `trow.yaml` file:

```
     ...
     containers:                                                               
      - name: trow-pod                                                          
        image: containersol/trow:default                                        
        args: ["-u", "myuser", "-p", "mypass", "-n", "trow:31000 trow.kube-public:31000"]                       
        imagePullPolicy: Always
     ...   

```

After this you will need to run `docker login` to push and pull images:

```
$ docker pull trow.test:8443/myimage
Using default tag: latest
Error response from daemon: Get https://trow.test:8443/v2/myimage/manifests/latest: unauthorized: authentication required
$ docker login trow.test:8443
Username: myuser
Password: 
Login Succeeded
$ docker pull ...
```

Trow also accepts a pointer to a file containing the password via the `--password-file` argument instead of `-p`. This allows the password to be stored in a Kubernetes secret that can be mounted into a volume inside the container.

### Troubleshooting

 - If you get an error when pushing, check the logs for the Trow pod e.g:

```
$ kubectl logs trow-deploy-5cf9bccdcc-g28vq -n kube-public
...
```

 - If a deployment isn't starting, check the logs for the replica set e.g:

```
$ kubectl get rs my-app-844d6db962
...
```

 - If there is a failed create message, the image may have been refused validation by Trow. If the message reads like:

```
Error creating: admission webhook "validator.trow.io" denied the request: *Remote* image docker.io/nginx disallowed as not contained in this registry and not in allow list
```

That means Trow considered the image name to refer to a _remote_ repository (i.e. not Trow itself) which has not been added to the allow list. If you believe the image should have been considered local, check the repository address appears in the list of addresses passed to Trow on start-up with the `-n` switch. If you want to allow a single remote image, add it to Trow by using the `--allow-images` flag. If you want to allow a whole repository or subdirectory of a repository use `--allow-prefixes`.

 - If the message reads:

```
Error creating: admission webhook "validator.trow.io" denied the request: Local image trow.kube-public:31000/notpresent disallowed as not contained in this registry and not in allow list
```

It means Trow expected to be able to serve this image itself but it wasn't found in the repository. Either push the image or use the `allow-images` or `allow-prefixes` flag to pre-approve images. Note that Kubernetes will keep trying to validate images.

 - If you get the error:

```
Error creating: Internal error occurred: failed calling admission webhook "validator.trow.io": Post https://trow.kube-public.svc:443/validate-image?timeout=30s: no endpoints available for service "trow"
```

Kubernetes has probably tried to start a new instance of Trow, but can't because there is no Trow instance to validate the image (can you say "catch 22"?). This will largely go away when we have HA, but until then you'll have to disable validation for Trow to restart. You'll also have to repeat the install steps for approving the certificate, distributing the certificate and setting up validation (in the future we will look to automate this process or reuse certificates to simply this).

 - If you get errors about certificates, either in `docker push` or from a replica set, you may need to re-approve and distribute the certificates (possibly due to the Trow pod being restarted):

```
$ kubectl certificate approve trow.kube-public
$ ./copy-certs.sh
$ ./sudo ./configure-host.sh --add-hosts
$ ./validate.sh
```

See above for full details.

## Install without TLS

Trow can be run with the `--no-tls` flag to serve over HTTP only. This can be
useful in development and testing, or when running on an internal, secure
network where the risks are understood.

The major problem is that the Docker client will not by default allow images to
be pushed or pulled without TLS. This can be circumvented in two ways:

 1) Using the localhost address for the registry.  

 2) By adding an "insecure-registries" entry to the Docker `daemon.json` file.
https://docs.docker.com/registry/insecure/

Method 1) can work well internally in a cluster using NodePort to forward
traffic. Method 2) can then be used to get an image into the registry from a
development machine.

### Microk8s and Containerd Support

For trow to work with containerd, we need to use the secure registry
configuration described here
https://github.com/containerd/cri/blob/master/docs/registry.md. This config is
only supported in containerd v1.3.0 and on. Currently (4 Oct 2019), microk8s
uses version 1.2.5 of containerd and is not compatible.

I am hopeful that Trow can use the mirror config in containerd to avoid the
need to edit `/etc/hosts`. This would be a large step forward, but does mean we
need access to the containerd config on all hosts. Once microk8s supports
containerd, I'll test out this theory.

