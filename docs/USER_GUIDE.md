# Trow User Guide

- [Trow User Guide](#trow-user-guide)
  - [Persisting Data/Images](#persisting-dataimages)
  - [Proxying other registries](#proxying-other-registries)
  - [Validating Webhook](#validating-webhook)
  - [Listing Repositories and Tags](#listing-repositories-and-tags)
  - [Multiplatform Builds](#multiplatform-builds)
  - [Troubleshooting](#troubleshooting)
    - [Where are the logs?](#where-are-the-logs)
    - [I can't push images into Trow](#i-cant-push-images-into-trow)
    - [My pod can't pull images from Trow](#my-pod-cant-pull-images-from-trow)
    - [Permission Denied Errors in Logs](#permission-denied-errors-in-logs)
    - [Errors When Pushing or Pulling Large Images](#errors-when-pushing-or-pulling-large-images)

More information is available in the [README](../README.md) and [Installation
instructions](../docs/HELM_INSTALL.md).

## Persisting Data/Images

If your cluster does not support Persistent Volumes, or you would like to use a different driver
(e.g. cephfs) you will need to manually assign a volume. This should be straightforward, but is
cluster-specific. Make sure that the volume is writeable by the Trow user (user id 333333 by
default). Normally this is taken care of by the `fsGroup` setting in the `securityContext` part of
the deployment, but this may not work for certain types of volume e.g. `hostPath` - in these
cases you may need to perform an explicit `chown` or `chmod` using the UID of the Trow user.

Backing up the Trow registry can be done by copying the data directory (`/data` by default).

## Proxying other registries

Trow will proxy any registry by default, ways to pull two syntaxes are supported:
* `podman pull mytrow/f/docker.io/nginx:latest` (custom)
* `curl https://mytrow/v2/nginx/manifests/latest?ns=docker.io` (standard registry mirror)

Special configuration (e.g. credentials) can be configured by using `--config-file`:

```yaml
# proxy.yaml
registry_proxies:
  registries:
    - host: my_custom_registry.example.com
      username: toto
      password: pass1234
```

```shell
$ RUST_LOG=info ./target/debug/trow
Starting Trow 0.8.0
Hostname of this registry: "[::]:8000"
Image validation webhook not configured
```

And then make the following request to the empty registry:

```shell
$ podman pull --tls-verify=false 127.0.0.1:8000/f/docker.io/nginx
Trying to pull 127.0.0.1:8000/f/docker.io/nginx:latest...
[...]
Writing manifest to image destination
2cd1d97f893f70cee86a38b7160c30e5750f3ed6ad86c598884ca9c6a563a501
```

Trow will keep a cached copy and check for new versions on each pull. The check is done via a HEAD
request which does not count towards the dockerhub rate limits. If the image cannot be pulled a cached
version will be returned, if available. This can be used to effectively mitigate availability issues with registries.

### Configuring containerd

See [the containerd docs](https://github.com/containerd/containerd/blob/main/docs/hosts.md#setup-default-mirror-for-all-registries).

```shell
$ tree /etc/containerd/certs.d
/etc/containerd/certs.d
└── _default
    └── hosts.toml

$ cat /etc/containerd/certs.d/_default/hosts.toml
[host."https://registry.example.com"]
  capabilities = ["pull", "resolve"]
```

Example bottlerocket initcontainer script:

```bash
#!/bin/sh
set -euo pipefail

IP_FAMILY=ipv6 # can also be local-ipv4
PORT=12345

IMDS_TOKEN="$(curl -s -X PUT -H "X-aws-ec2-metadata-token-ttl-seconds: 360" "http://[fd00:ec2::254]/latest/api/token")"
IP="$(curl -H "X-aws-ec2-metadata-token: $IMDS_TOKEN" http://[fd00:ec2::254]/latest/meta-data/${IP_FAMILY})"
cat >> /.bottlerocket/rootfs/etc/containerd/config.toml << EOF
[plugins."io.containerd.grpc.v1.cri".registry]
  config_path = "/etc/containerd/certs.d"
EOF
mkdir -p /.bottlerocket/rootfs/etc/containerd/certs.d/_default/
chmod -R 0755 /.bottlerocket/rootfs/etc/containerd/certs.d/
cat >> /.bottlerocket/rootfs/etc/containerd/certs.d/_default/hosts.toml << EOF
[host."http://${IP}:${PORT}"]
  capabilities = ["pull", "resolve"]
  skip_verify = true
EOF
chmod 0644 /.bottlerocket/rootfs/etc/containerd/certs.d/_default/hosts.toml
chown -R root /.bottlerocket/rootfs/etc/containerd/certs.d/
```

TODO: cri-o configuration (https://github.com/cri-o/cri-o/discussions/9383).

## Validating Webhook

The validating webhook can be configured using `--image-validation-config-file` argument like so:

```yaml
# validation.yaml
image_validation:
  default: Deny
  allow:
    - my-trow-domain.trow.io/
    - k8s.gcr.io/
  deny:
    - my-trow-domain.trow.io/my-secret-image
```

```shell
$ ./trow --config-file ./validation.yaml
Starting Trow 0.6.0 on 0.0.0.0:8000
Hostname of this registry (for the MutatingWebhook): "0.0.0.0"
Image validation webhook configured:
  Default action: Deny
  Allowed prefixes: ["my-trow-domain.trow.io/", "k8s.gcr.io/"]
  Denied prefixes: ["my-trow-domain.trow.io/my-secret-image"]
Proxy registries not configured
```

## Listing Repositories and Tags

Trow implements the [OCI Distribution
Specification](https://github.com/opencontainers/distribution-spec/blob/main/spec.md) which
includes API methods for listing repositories and tags. Unfortunately the Docker CLI doesn't support
these endpoints, so we need to use a third-party tool. It is possible to use curl, but this gets
complicated when dealing with password protected registries, so we recommend the [docker-ls
tool](https://github.com/mayflower/docker-ls).

Using `docker-ls` is fairly straightforward, for example, to list all repositories in a registry:

```shell
$ docker-ls repositories -u myuser -p mypass -r https://registry.trow.io
requesting list . done
repositories:
- alpine
- one/two
- user1/web
- user2/web
```

To list all tags for a repository:

```shell
$ docker-ls tags user1/web -u myuser -p mypass -r https://registry.trow.io
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

Trow has builds for amd64 and arm64. Images tagged `latest` or `default` are currently amd64 only.

If there's another build you would like to see, please get in contact.

## Troubleshooting

### Where are the logs?

The first place to look for debugging information is in the output from the
`kubectl describe` command. It's worth looking at the output for the deployment,
replicaset and pod. Assuming the namespace for the Trow is "trow":

```shell
$ kubectl describe deploy -n trow trow-deploy
$ kubectl describe replicaset -n trow trow-deploy
$ kubectl describe pod -n trow trow-deploy
```

In particular, look for problems pulling images or with containers crashing.

For the actual application logs try:

```shell
$ kubectl logs -n trow trow-deploy-596bf849c8-m7b7l
```

The ID at the end of your pod name will be different, but you should be able to
use autocomplete to get the correct name (hit the tab key after typing
"trow-deploy").

If there are no logs or you get output like:

```log
Error from server (BadRequest): container "trow-pod" in pod "trow-deploy-6f6f8fbc6d-rndtd" is waiting to start: PodInitializing
```

Look at the logs for the init container:

```shell
$ kubectl logs -n trow trow-deploy-596bf849c8-m7b7l -c trow-init
```

### I can't push images into Trow

If it seems like you can connect to Trow successfully but then uploads fail with `manifest invalid`
or `Internal Server Error`, Trow may be having trouble saving to the filesystem. First check
the logs (see "Where are the logs?" above). If this is the case, check there is free space on the
volume and the Trow user has the correct privileges to write to the volume. In particular, verify
that the settings for the volume match the UID of the Trow user (333333 by default):

```yaml
# ...
    spec:
      containers:
      - name: trow
      # ...
      securityContext:
        runAsUser: 333333
        runAsGroup: 333333
        fsGroup: 333333
```

### My pod can't pull images from Trow

If a deployment isn't starting, check the logs for the replica set e.g:

```bash
kubectl get rs my-app-844d6db962
```

* If you get the error:

  ```log
  Error creating: Internal error occurred: failed calling admission webhook "validator.trow.io": Post https://trow.kube-public.svc:443/validate-image?timeout=30s: no endpoints available for service "trow"
  ```

  Trow probably isn't running and the webhook is configured to `Fail` on error.

* If the message reads like:

  ```log
  Error creating: admission webhook "validator.trow.io" denied the request: my_registry.io/nginx: Image is neither explicitly allowed nor denied (using default behavior)
  ```

  That means:

  1. The validation webhook is active
  2. `my_registry.io/` has not been added to the allow list
  3. The default behavior is configured to `"Deny"`


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
