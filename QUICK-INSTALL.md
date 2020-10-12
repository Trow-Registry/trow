Quick Install for Dev/Test Instructions
=======================================

We use `nix` to make your live easier. `nix` is a package management ecosystem for reproducible
builds. We use it for reproducible dev environments that come bundled with all required tools.

_For anything different than short-lived dev or testing clusters, please see the [standard
installation instructions](install/INSTALL.md)._

## Prerequisistes

- *nix machine &mdash; windows is currenlty only supported partially
- `nix` package manager installed
- this repository cloned

To install `nix`:

```console
curl -L https://nixos.org/nix/install | sh
```

## Enter dev environment

To enter the dev entironment, at the root of this repo, run:

```console
nix-shell
```
_First time, this takes a while and uses aproximately 500Mb of disk space. The tradeoff at cost
of your disk space and at the benefit of your convenience is deliberate. We hope you agree._


Exit normally, as you would from any shell, with `CTRL+D`.


## Dev `menu` & `just-menu`

Within the dev environment, first enter de `./dev` folder, then run and review the output of
the following commands to understand the current state of convenience repo tooling:

- `menu`
- `just-menu` (this is the conctext sensitive one to invoking from within `./dev` folder)


## Quick-Install

Within the dev environment, from within the `./dev` folder:

1. just run:  `just quick-install`.
2. activate local static dns with `dns-activate`.
3. install local root CA with `ca-install`.


## Test it out

Trow has configured the domain `registry.local` to point to your trow instance within the
kubernetes cluster. Try pushing an image:

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
$ kubectl apply -f install/trow.yaml 

serviceaccount/trow unchanged
role.rbac.authorization.k8s.io/trow unchanged
clusterrole.rbac.authorization.k8s.io/trow unchanged
rolebinding.rbac.authorization.k8s.io/trow unchanged
clusterrolebinding.rbac.authorization.k8s.io/trow unchanged
deployment.apps/trow-deploy configured
service/trow unchanged
```


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

## Uninstall

To tear down the dev evironment (including the dev cluster), just run:

```console
just tear-down
```

To uninstall local CA and disable local static dns run:

1. `ca-uninstall`
2. `dns-disable`

## Troubleshooting

See the [User Guide](docs/USER_GUIDE.md#Troubleshooting)

