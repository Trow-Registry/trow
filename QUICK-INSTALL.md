Quick Install for Dev/Test Instructions
=======================================

## Install with TLS

These instructions are intended for installing Trow on short-lived clusters, typically when
developing or testing with Kubernetes. For longer-lived clusters, please see the [standard
installation instructions](install/INSTALL.md). 

The install script will configure the Trow registry on an existing Kubernetes
cluster, with a certificate signed by the Kubernetes CA. It will copy the certificate to the nodes
in the Kubernetes cluster as well the client machine. The address "trow.kube-public" is configured
to point to the registry by adding an entry to `/etc/hosts`.

The script has been primarily tested with GKE and minikube on MacOS and Linux. It isn't currently
compatible with containerd based distributions such as microk8s (see #14).

***These instructions modify nodes in your cluster. Only run on test or dev clusters.***

### Pre-requisites

 - `kubectl` is installed and configured to point at the cluster you wish to install Trow on
 - You've cloned or downloaded this repo
 - Port 31000 can be reached on the worker nodes. You may need to edit the network policy or firewall settings if running in the cloud. For example, run the following if using GKE:
```
$ gcloud compute firewall-rules create trow --allow tcp:31000 --description "Allow inbound Trow registry traffic" --project <project name>
```
 - If you're running on GKE or have RBAC configured you may need to expand your
   rights to be able to create the needed service-account (on GKE the user is probably your e-mail address):
 - The cluster role binding must be created each time you install trow on a new cluster.
```
$ kubectl create clusterrolebinding cluster-admin-binding --clusterrole=cluster-admin --user=<user>
clusterrolebinding.rbac.authorization.k8s.io "cluster-admin-binding" created
```

### Automatic installation

 - Just run `./install.sh` from the `quick-install` directory and follow the prompts. 
 - You can also specify installation namespace by running `./install.sh <namespace>`
 - If you are using a Mac, restart Docker once the install script has completed.

If you'd rather have more control over the process, follow the [manual
steps](./quick-install/MANUAL_INSTALL.md).

### Test it out

Trow has configured the domain `trow.kube-public` to point to your kubernetes cluster. Try pushing an image:

```
$ docker pull nginx:alpine
alpine: Pulling from library/nginx
Digest: sha256:e0292d158b6b353fde34909243a4886977cb9d1abb8a8a5fef9e0ff7138dd3e2
Status: Image is up to date for nginx:alpine
```
```
$ docker tag nginx:alpine trow.kube-public:31000/test/nginx:alpine
```
```
$ docker push trow.kube-public:31000/test/nginx:alpine
The push refers to repository [trow.kube-public:31000/test/nginx]
979531bcfa2b: Pushed 
8d36c62f099e: Pushed 
4b735058ece4: Pushed 
503e53e365f3: Pushed 
alpine: digest: sha256:bfddb36c23addfd10db511d95b7508fa7b6b2aca09b313ff3ef73c3752d11a55 size: 11903
```

If the push seems to hang, check if port 31000 is blocked (common with cloud provider default network rules).

If you're using Google cloud, you can open port 31000 as follows:

```
$ gcloud compute firewall-rules create trow-rule --allow=tcp:31000
```

The Kubernetes cluster should now be able to pull and run the image:

```
$ kubectl create deploy trow-test --image=trow.kube-public:31000/test/nginx:alpine
deployment.apps "trow-test" created
```
```
$ kubectl get deploy trow-test
NAME        DESIRED   CURRENT   UP-TO-DATE   AVAILABLE   AGE
trow-test   1         1         1            1           8s
```

If you have enabled validation of images, try running a Docker Hub image, which should be denied:

```
$ kubectl create deploy proxy --image=docker.io/nginx
deployment.apps "proxy" created
```
```
$ kubectl get deployment proxy
NAME      DESIRED   CURRENT   UP-TO-DATE   AVAILABLE   AGE
proxy     1         0         0            0           13s
```
```
$ kubectl describe rs proxy
...
  Warning  FailedCreate  16s (x13 over 57s)  replicaset-controller  Error creating: admission webhook "validator.trow.io" denied the request: Remote image docker.io/nginx disallowed as not contained in this registry and not in allow list
```

If you want to allow images from the Docker Hub, take a look at the `--allow-docker-official` and `--allow-prefixes` arguments. This can be passed to Trow via the `trow.yaml` file.

The following example allows official images from Docker Hub and images with the prefix "registry.container-solutions.com/" to run in the cluster:
```
containers:
- name: trow-pod
  image: containersol/trow:default
  args: ["-n", "trow:31000 trow.kube-public:31000", "-c", "/certs/domain.crt","--allow-docker-official","--allow-prefixes","registry.container-solutions.com/"]
```
To apply the changes and restart Trow, run the following:

```
$ sed "s/{{namespace}}/kube-public/" install/trow.yaml | kubectl apply -f -

serviceaccount/trow unchanged
role.rbac.authorization.k8s.io/trow unchanged
clusterrole.rbac.authorization.k8s.io/trow unchanged
rolebinding.rbac.authorization.k8s.io/trow unchanged
clusterrolebinding.rbac.authorization.k8s.io/trow unchanged
deployment.apps/trow-deploy configured
service/trow unchanged
```

This assumes you installed Trow to the default namespace of `kube-public`. If you installed Trow to
a different namespace than `kube-public`, replace as appropriate.

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
```
```
$ docker login trow.test:8443
Username: myuser
Password: 
Login Succeeded
$ docker pull ...
```

Trow also accepts a pointer to a file containing the password via the `--password-file` argument instead of `-p`. This allows the password to be stored in a Kubernetes secret that can be mounted into a volume inside the container.

### Uninstall

To uninstall the Trow Kubernetes components run the following from the `quick-install` directory:

```
sed "s/{{namespace}}/kube-public/" trow.yaml | kubectl delete -f -
```

This assumes you installed Trow to the default namespace of `kube-public`. If you installed Trow to
a different namespace than `kube-public`, replace as appropriate.

If you enabled validation, remove the webhook with:

```
kubectl delete -f validate.yaml
```

This will not remove the Trow certificates or revert the changes to `/etc/hosts` on the nodes or
client. Leaving these changes shouldn't cause any problems. If you would like to remove them, the
lines in `/etc/hosts` are clearly marked, and the certs should be found under `/etc/docker/certs.d/`
on both the the nodes and client.

### Troubleshooting

See the [User Guide](docs/USER_GUIDE.md#Troubleshooting)

