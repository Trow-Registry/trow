Installation Instructions
=========================

## Install with TLS

***These instructions modify nodes in your cluster. Only run on test clusters currently.***

The following instructions install the Trow registry on Kubernetes, with a
certificate signed by the Kubernetes CA. They have been tested on both minikube
(with the KVM2 driver on Linux) and GKE.

 - If you're running on GKE or have RBAC configured you may need to expand your
   rights to be able to create the needed service-account (on GKE the user is probably your e-mail address):
```
$ kubectl create clusterrolebinding cluster-admin-binding --clusterrole=cluster-admin --user=<user>
```
 - Run the main k8s yaml:

```
$ kubectl create -f trow.yaml
```

 - This will create a service for Trow that includes a NodePort for external
   access (if you don't want this, edit `trow.yaml`). It will also pull the Trow
image and start up the pod, which may take a moment to download. The Trow pod
will then get stuck in init, waiting for us to approve its certificate. Do this
by:

```
$ kubectl certificate approve trow.kube-public
```

 - If you get the error "No resources found" wait a moment and try again. In some
cases it takes a few minutes for the request to appear in k8s. 
 - Trow should now be up and running, but we still need to make the nodes trust
   the certificate if we want them to be able to pull. The easy way is by
running the following script, but be aware that this will modify files on the
Nodes, including /etc/hosts:

```
$ cd install
$ ./copy-certs.sh
```

Note there is an issue with this approach, as new nodes will not automatically
get the certs and will be unable to pull from Trow. We hope to have a better
solution in the future, but it may require changes to Kubernetes.

 - Finally, you probably want to be able to push from your development laptop,
   which you can do with:

```
$ sudo ./configure-host.sh --add-hosts
```

This will copy Trow's cert into Docker and also add an entry to /etc/hosts for
trow.kube-public. We can test it all out by trying to push an image:

```
$ docker pull nginx:alpine
$ docker tag nginx:alpine trow.kube-public:31000/test/nginx:alpine
$ docker push trow.kube-public:31000/test/nginx:alpine
```

If the push seems to hang, check if port 31000 is blocked (in GKE it normally is
by default).

The Kubernetes cluster should now be able to pull and run the image:

```
$ kubectl run trow-test --image=trow.kube-public:31000/test/nginx:alpine
$ kubectl get deploy trow-test
```

### Troubleshooting

If you get an error when pushing, check the logs for the Trow pod e.g:

```
$ kubectl logs trow-deploy-5cf9bccdcc-g28vq -n kube-public
```

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

