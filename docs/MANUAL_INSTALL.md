# Manual Installation

The following walks through the steps in the automatic installer. If you need to
customise Trow for any reason, this is a good place to start.

 - Apply the `trow.yaml` file from the quick-install directory:

```
$ cd quick-install
$ kubectl apply -f trow.yaml
serviceaccount "trow" created
role.rbac.authorization.k8s.io "trow" created
clusterrole.rbac.authorization.k8s.io "trow" created
rolebinding.rbac.authorization.k8s.io "trow" created
clusterrolebinding.rbac.authorization.k8s.io "trow" created
deployment.apps "trow-deploy" created
service "trow" created
```

 - This will create a service for Trow that includes a NodePort for external
   access (if you don't want this, edit `trow.yaml`). It will also pull the Trow
image and start up the pod, which may take a moment to download. The Trow pod
will then get stuck in init, waiting for us to approve its certificate. Do this
by:

```
$ kubectl certificate approve trow.kube-public
certificatesigningrequest.certificates.k8s.io "trow.kube-public" approved
```

 - If you get the error "No resources found" wait a moment and try again. In some
cases it takes a few minutes for the request to appear in k8s. 
 - Trow should now be up and running, but we still need to make the nodes trust
   the certificate if we want them to be able to pull. The easy way is by
running the following script, but be aware that this will modify files on the
Nodes, including `/etc/hosts`:

```
$ cd install
$ ./copy-certs.sh
Copying certs to nodes
job.batch "copy-certs-5a2fa2bc-3457-11e9-a2bc-42010a800018" created
job.batch "copy-certs-55cf8134-3457-11e9-a2bc-42010a800018" created
```

Note there is an issue with this approach, as new nodes will not automatically
get the certs and will be unable to pull from Trow. We will fix this issue in the future.

 - Finally, you probably want to be able to push images from your development laptop,
   which you can do with:

```
$ ./configure-host.sh --add-hosts
Copying cert into Docker
Successfully copied cert
Adding entry to /etc/hosts for trow.kube-public

Exposing registry via /etc/hosts

Successfully configured localhost
```

This will copy Trow's cert into Docker and also add an entry to /etc/hosts for
trow.kube-public. 

One of the major features of Trow is the ability to control the images that run
in the cluster. To achieve this, we need to set-up an [Admission
Webhook](https://kubernetes.io/docs/reference/access-authn-authz/extensible-admission-controllers/#admission-webhooks)
in the Kubernetes cluster that will ask Trow for permission everytime a resource
is created or updated.

The default policy will allow all images local to the Trow registry to be used, plus
Kubernetes system images and the Trow images themselves. All other images are denied by
default, including Docker Hub images.

To enable validation run (from the `install` directory):

```
$ ./validate.sh 
Setting up trow as a validating webhook
WARNING: This will limit what images can run in your cluster

validatingwebhookconfiguration.admissionregistration.k8s.io "trow-validator" configured
```

You can test everything worked correctly by following the instructions in [the
main install guide](../INSTALL.md).
