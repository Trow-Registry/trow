# Trow User Guide

- [Trow User Guide](#trow-user-guide)
  - [Persisting Data/Images](#persisting-dataimages)
  - [Proxying other registries (and MutatingWebhook)](#proxying-other-registries-and-mutatingwebhook)
  - [Validating Webhook](#validating-webhook)
    - [Configuration](#configuration)
    - [Troubleshooting](#troubleshooting)
  - [Listing Repositories and Tags](#listing-repositories-and-tags)
  - [Multiplatform Builds](#multiplatform-builds)
  - [Troubleshooting](#troubleshooting-1)
    - [Where are the logs?](#where-are-the-logs)
    - [I can't push images into Trow](#i-cant-push-images-into-trow)
    - [My pod can't pull images from Trow](#my-pod-cant-pull-images-from-trow)
    - [Permission Denied Errors in Logs](#permission-denied-errors-in-logs)
    - [Errors When Pushing or Pulling Large Images](#errors-when-pushing-or-pulling-large-images)

More information is available in the [README](../README.md) and [Installation
instructions](../docs/KUSTOMIZE_INSTALL.md).

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
cluster-specific. Make sure that the volume is writeable by the Trow user (user id 333333 by
default). Normally this is taken care of by the `fsGroup` setting in the `securityContext` part of
the deployment YAML, but this may not work for certain types of volume e.g. `hostPath` - in these
cases you may need to perform an explicit `chown` or `chmod` using the UID of the Trow user.

Backing up the Trow registry can be done by copying the data directory (`/data` by default).

## Proxying other registries (and MutatingWebhook)

Trow can be configured as a proxy cache for other registries by passing the argument
`--proxy-registry-config-file` on start-up. Any repositories under `f/{alias}/` will automatically be pulled
from the matching registry. For example, if we start Trow with:

```yaml
# cfg.yaml
- alias: docker
  host: registry-1.docker.io
- alias: my-custom-registry
  host: my_custom_registry.example.com
  username: toto
  password: pass1234
```

```
$ trow --proxy-registry-config-file ./cfg.yaml
Starting Trow 0.3.5 on 0.0.0.0:8000

Maximum blob size: 8192 Mebibytes
Maximum manifest size: 4 Mebibytes
Hostname of this registry (for the MutatingWebhook): "0.0.0.0"
Image validation webhook not configured
Proxy registries configured:
  - docker: docker.io
  - quay: quay.io
  - nvcr: nvcr.io
Trow is up and running!
```

And then make the following request to the empty registry:

```
$ docker pull localhost:8443/f/docker/nginx:latest
latest: Pulling from f/docker/nginx
bb79b6b2107f: Already exists
5a9f1c0027a7: Pull complete
b5c20b2b484f: Pull complete
166a2418f7e8: Pull complete
1966ea362d23: Pull complete
Digest: sha256:34f3f875e745861ff8a37552ed7eb4b673544d2c56c7cc58f9a9bec5b4b3530e
Status: Downloaded newer image for localhost:8443/f/docker/nginx:latest
localhost:8443/f/docker/nginx:latest
```

Trow will keep a cached copy and check for new versions on each pull. The check is done via a HEAD
request which does not count towards the dockerhub rate limits. If the image cannot be pulled a cached
version will be returned, if available. This can be used to effectively mitigate availability issues with registries.

The helm chart contains a `MutatingWebhookConfiguration`  that will automatically rewrite pod specs to pull through Trow.

## Validating Webhook

### Configuration

The validating webhook can be configured using `--image-validation-config-file` argument like so:

```yaml
# validation.yaml
default: Deny
allow:
  - my-trow-domain.trow.io/
  - k8s.gcr.io/
deny:
  - my-trow-domain.trow.io/my-secret-image
```

```console
$ ./trow --image-validation-config-file ./validation.yaml
Starting Trow 0.3.5 on 0.0.0.0:8000

Maximum blob size: 8192 Mebibytes
Maximum manifest size: 4 Mebibytes
Hostname of this registry (for the MutatingWebhook): "0.0.0.0"
Image validation webhook configured:
  Default action: Deny
  Allowed prefixes: ["my-trow-domain.trow.io/", "k8s.gcr.io/"]
  Denied prefixes: ["my-trow-domain.trow.io/my-secret-image"]
Proxy registries not configured
Trow is up and running!
```

### Troubleshooting

If a deployment isn't starting, check the logs for the replica set e.g:

```bash
kubectl get rs my-app-844d6db962
```

If there is a failed create message, the image may have been refused validation by Trow. If the message reads like:

```
Error creating: admission webhook "validator.trow.io" denied the request: my_registry.io/nginx: Image is neither explicitly allowed nor denied (using default behavior)
```

That means:
1. The validation webhook is active
2. `my_registry.io/` has not been added to the allow list
3. The default behavior is configured to `"Deny"`


Otherwise, if the error reads like:

```
Error creating: Internal error occurred: failed calling admission webhook "validator.trow.io": Post https://trow.kube-public.svc:443/validate-image?timeout=30s: no endpoints available for service "trow"
```

Trow probably isn't running and the webhook is configured to `Fail` on error. You will need to disable the admission webhook (or, for helm chart: `onWebhookFailure: Ignore`) and restart Trow.

## Listing Repositories and Tags

Trow implements the [OCI Distribution
Specification](https://github.com/opencontainers/distribution-spec/blob/main/spec.md) which
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

## Multiplatform Builds

Trow has builds for amd64, armv7 and arm64. Images with a release version but no explicit platform e.g. `trow:0.3` or `trow:0.3.2` should be _multiplatform_ images that will automatically pull the correct version of the image for the current platform. Images tagged `latest` or `default` are currently amd64 only. Images should be pushed to both [GHCR](https://github.com/orgs/extrality/packages/container/package/trow%2Ftrow) and the [Docker Hub](https://hub.docker.com/r/containersol/trow).

If there's another build you would like to see, please get in contact.

## Troubleshooting

### Where are the logs?

The first place to look for debugging information is in the output from the
`kubectl describe` command. It's worth looking at the output for the deployment,
replicaset and pod. Assuming the namespace for the Trow is "trow":

```
$ kubectl describe deploy -n trow trow-deploy
$ kubectl describe replicaset -n trow trow-deploy
$ kubectl describe pod -n trow trow-deploy
```

In particular, look for problems pulling images or with containers crashing.

For the actual application logs try:

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

### I can't push images into Trow

If you get an error like:

```
$ docker push trow.kube-public:31000/nginx:alpine
The push refers to repository [trow.kube-public:31000/nginx]
Get https://trow.kube-public:31000/v2/: dial tcp 192.168.39.211:31000: connect: no route to host
```

Your client isn't reaching the Trow service. Please check the following:

 - Verify that Trow is running (e.g. `kubectl get deploy -n trow trow-deploy`).
   If not, refer to the section on logs above to diagnose the issue.
 - Check that a service exists for Trow (e.g. `kubectl describe svc -n trow trow`).
 - Check that your network or cloud provider isn't blocking access.


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

This indicates it can't resolve the host name. Running `install/configure-host.sh` should add an
entry to `/etc/hosts` that will fix the issue.

If it seems like you can connect to Trow successfully but then uploads fail with `manifest invalid`
or `Internal Server Error`, Trow may be having trouble saving to the filesystem. First check
the logs (see "Where are the logs?" above). If this is the case, check there is free space on the
volume and the Trow user has the correct privileges to write to the volume. In particular, verify
that the settings for the volume match the UID of the Trow user (333333 by default):

```yaml
# ...
    spec:
      containers:
      - name: trow-pod
        image: containersol/trow:0.3
      # ...
      securityContext:
        runAsUser: 333333
        runAsGroup: 333333
        fsGroup: 333333
```

### My pod can't pull images from Trow

If you get the error:

```
Error creating: Internal error occurred: failed calling admission webhook "validator.trow.io": Post https://trow.kube-public.svc:443/validate-image?timeout=30s: no endpoints available for service "trow"
```

Trow probably isn't running and the webhook is configured to `Fail` on error. You will need to disable the admission webhook (or, for helm chart: `onWebhookFailure: Ignore`) and restart Trow.


```
The push refers to repository [trow.kube-public:31000/test/nginx]
Get https://trow.kube-public:31000/v2/: x509: certificate signed by unknown authority
```

If you get this error, and you are using the quick install on Docker for Mac, try restarting Docker.

### Permission Denied Errors in Logs

If you get errors such as `{ code: 13, kind: PermissionDenied, message: "Permission denied" }`, it is
possible that Trow can't write to the data directory. Please verify that the data volume is
accessible and writeable by the Trow user. If not, please use `chown` or `chmod` to give the Trow
user access. As the Trow user only exists in the container, you will likely need to use it's
equivalent UID e.g. `chown 333333 /data`.

### Errors When Pushing or Pulling Large Images

If you get errors when dealing with large images, but not with smaller images, you may need to
configure your ingress to explicitly allow large transfers. For example, if you are using the
NGINX ingress, add the following annotation to the Kubernetes configuration:

```yaml
nginx.ingress.kubernetes.io/proxy-body-size: "0"
```
