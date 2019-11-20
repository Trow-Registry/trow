# Trow User Guide

 * [Persisting Data/Images](#persisting-dataimages)
 * [Listing Repositories and Tags](#listing-repositories-and-tags)
 * [Using Curl Securely](#using-curl-securely)
 * [Troubleshooting](#troubleshooting)

More information is available in the [README](../README.md) and [Installation
instructions](../install/INSTALL.md).

## Persisting Data/Images

If you are using the quick install, note that Trow will store images and metadata in a Kubernetes
[emptyDir volume](https://kubernetes.io/docs/concepts/storage/volumes/#emptydir). This means that
the data will survive pod restarts, but will be lost if the Trow pod is deleted or evicted from the
node it is running on. This can occur when a node fails or is brought down for maintenance.

The standard install initialises a Kubernetes [Persistent
Volume](https://kubernetes.io/docs/concepts/storage/persistent-volumes/), as a permanent store of
data. This should be reattached in the case of node or pod failure, thus avoiding data loss.

If your cluster does not support Persistent Volumes, or you would like to use a different driver
(e.g. cephfs) you will need to manually assign a volume. This should be straightforward, but is
cluster-specific. Make sure that the volume is writeable by the Trow user (user id 999 by
default).

Backing up the Trow registry can be done by copying the `/data` directory. 

## Listing Repositories and Tags

Trow implements the [OCI Distribution
Specification](https://github.com/opencontainers/distribution-spec/blob/master/spec.md) which
includes API methods for listing repositories and tags. Unfortunately the Docker CLI doesn't support
these endpoints, so we need to use a third-party tool. It is possible to use curl, but this gets
complicated when dealing with password protected registries, so we recommend the [docker-ls
tool](https://github.com/mayflower/docker-ls).

Using `docker-ls` is fairly straightforward, for example, to list all repositories in a registry:

```
docker-ls repositories -u myuser -p mypass -r https://registry.trow.io
requesting list . done
repositories:
- alpine
- one/two
- user1/web
- user2/web
```

To list all tags for a repository:

```
docker-ls tags user1/web -u myuser -p mypass -r https://registry.trow.io
requesting list . done
repository: user1/web
tags:
- default
- test
```

If you want to play with the underlying APIs, the URL for listing repositories is `/v2/_catalog` and
the tags for any given repository can be listed with `/v2/<repository_name>/tags/list`.

The catalog endpoint is a matter of debate by the OCI and may be replaced in future versions.  Do
not expect different registries to have compatible implementations of this endpoint for historical
reasons and ambiguities in specification.

## Troubleshooting

### Where are the logs?

The first place to look for debugging information is in the output from the
`kubectl describe` command. It's worth looking at the output for the deployment,
replicaset and pod. Assuming the namespace for the Trow is "trow" (if you used the quick-install it
will be kube-public):

```
$ kubectl describe deploy -n trow trow-deploy
$ kubectl describe replicaset -n trow trow-deploy
$ kubectl describe pod -n trow trow-deploy
```

In particular, look for problems pulling images or with containers crashing.

For the actual applcation logs try:

```
$ kubectl logs -n trow trow-deploy-596bf849c8-m7b7l
```

The ID at the end of your pod name will be different, but you should be able to
use autocomplete to get the correct name (hit the tab key after typing
"trow-deploy").

If there are no logs or you get output like: 

```
Error from server (BadRequest): container "trow-pod" in pod "trow-deploy-6f6f8fbc6d-rndtd" is waiting to start: PodInitializing
```

Look at the logs for the init container:

```
$ kubectl logs -n trow trow-deploy-596bf849c8-m7b7l -c trow-init
```

If you used the quick-install, the `copy-certs` job may also log errors:

```
$ kubectl logs -n kube-public copy-certs-925a5126-48bd-43d4-b9ea-3f792519b051-fznp8
```

### I can't push images into Trow

If you get an error like:

```
$ docker push trow.kube-public:31000/nginx:alpine
The push refers to repository [trow.kube-public:31000/nginx]
Get https://trow.kube-public:31000/v2/: dial tcp 192.168.39.211:31000: connect: no route to host
```

Your client isn't reaching the Trow service. Please check the following:

 - Verify that Trow is running (e.g. `kubectl get deploy -n trow
   trow-deploy`). If not, refer to the section on logs above to diagnose the
   issue. 
 - Check that a service exists for Trow (e.g. `kubectl describe svc -n
   trow trow`). 
 - Check that your network or cloud provider isn't blocking access. 

_The rest of the advice in this question is applicable only to the quick-install_

 - Ensure port 31000 is accessible. This will likely mean editing network rules if using a public
   cloud.
 - Make sure that your client is pointing to the correct address. The IP address given in the error
   message should match the public IP of one of the cluster nodes. If it doesn't, try running the
   `install/configure-host.sh` script.

If you get an error like:

```
$ docker push trow.kube-public:31000/nginx:alpine
The push refers to repository [trow.kube-public:31000/nginx]
Get https://trow.kube-public:31000/v2/: x509: certificate signed by unknown authority
```

This indicates the Docker client doesn't trust the remote server. To fix this,
we need to add Kubernetes CA certificate or the Trow certificate to Docker. The
easiest way to do this is by running the `install/configure-host.sh`, which
should place the correct under `/etc/docker/certs.d/_registry-name_`.  

If you get an error like:

```
docker push trow.kube-public:31000/nginx:alpine
The push refers to repository [trow.kube-public:31000/nginx]
Get https://trow.kube-public:31000/v2/: dial tcp: lookup trow.kube-public: No address associated with hostname
```

This indicates it can't resolve the host name. Running `install/configure-host.sh` should add an entry to `/etc/hosts` that will fix the issue.

### My pod can't pull images from Trow

If a deployment isn't starting, check the logs for the replica set e.g:

```
$ kubectl get rs my-app-844d6db962
...
```

If there is a failed create message, the image may have been refused validation by Trow. If the message reads like:

```
Error creating: admission webhook "validator.trow.io" denied the request: *Remote* image docker.io/nginx disallowed as not contained in this registry and not in allow list
```

That means Trow considered the image name to refer to a _remote_ repository (i.e. not Trow itself)
which has not been added to the allow list. If you believe the image should have been considered
local, check the repository address appears in the list of addresses passed to Trow on start-up with
the `-n` switch. If you want to allow a single remote image, add it to Trow by using the
`--allow-images` flag. If you want to allow a whole repository or subdirectory of a repository use
`--allow-prefixes`.

If the message reads:

```
Error creating: admission webhook "validator.trow.io" denied the request: Local image trow.kube-public:31000/notpresent disallowed as not contained in this registry and not in allow list
```

It means Trow expected to be able to serve this image itself but it wasn't found in the repository.
Either push the image or use the `allow-images` or `allow-prefixes` flag to pre-approve images. Note
that Kubernetes will keep trying to validate images.

If you get the error:

```
Error creating: Internal error occurred: failed calling admission webhook "validator.trow.io": Post https://trow.kube-public.svc:443/validate-image?timeout=30s: no endpoints available for service "trow"
```

Trow probably isn't running. You will need to disable the admission webhook and restart Trow. To
disable the webhook run `kubectl delete validatingwebhookconfigurations.admissionregistration.k8s.io
trow-validator`. If Trow doesn't restart automatically, refer to the other sections on
troubleshooting or try reinstalling.

If the error is not to do with validation and you used the quick-install, it may be that the node is
unable to pull from the Trow registry. By default nodes are configured by the `copy-certs` job. You
can check that the job completed succesfully with `kubectl get jobs -n kube-public`. If the node is
new, try running the script `install/copy-certs.sh`.


```
The push refers to repository [trow.kube-public:31000/test/nginx]
Get https://trow.kube-public:31000/v2/: x509: certificate signed by unknown authority
```

If you get this error, and you are using the quick install on Docker for Mac, try restarting Docker.
