Installation Instructions
=========================

***These instructions modify nodes in your cluster. Only run on test clusters currently.***

To install the trow registry on Kubernetes, with a certificate signed by the k8s
CA:

 - If you're running on GKE or have RBAC configured you may need to expand your
   rights to be able to create the needed service-account:
```
$ kubectl create clusterrolebinding cluster-admin-binding --clusterrole=cluster-admin --user=<user>
```
 - Run the main k8s yaml:

```
$ kubectl create -f trow.yaml
```

 - This will create a service for Trow that includes a NodePort for external
   access (if you don't want this, edit `trow.yaml`). It will also start up the
trow pod which will get stuck in init, waiting for us to approve it's
certificate. Do this by:

```
$ kubectl certificate approve trow.kube-public
```

 - Trow should now be up and running, but we still need to make the nodes trust
   the certificate if we want them to be able to pull. The easy way is by
running the following script, but be aware that this will modify files on the
Nodes, including /etc/hosts:

```
$ ./copy-certs.sh
```

Note there is an issue with this approach, as new nodes will not automatically
get the certs and will be unable to pull from Trow. It would be much easier if
Kubernetes nodes automatically trusted the k8s certificate and were connected to
the Kubernetes DNS. 

 - Finally, you probably want to be able to push from your development laptop,
   which you can do with:

```
$ sudo ./configure_host.sh --add-hosts
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

And test we can pull in Kubernetes:

```
$ kubectl run trow-test --image=trow.kube-public:31000/test/nginx:alpine
$ kubectl get deploy trow-test
```





