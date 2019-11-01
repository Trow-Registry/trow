# Trow User Guide

 * [Persisting Data/Images](#persisting-dataimages)
 * [Listing Repositories and Tags](#listing-repositories-and-tags)
 * [Using Curl Securely](#using-curl-securely)
 * [Troubleshooting](#troubleshooting)

More information is available in the [README](../README.md) and [Installation
instructions](../INSTALL.md).

## Persisting Data/Images

By default, Trow stores images and metadata in a Kubernetes [emptyDir
volume](https://kubernetes.io/docs/concepts/storage/volumes/#emptydir). This
means that the data will survive pod restarts, but will be lost if the Trow pod
is deleted or evicted from the node it is running on. This can occur when a node
fails or is brought down for maintenance.

To avoid losing data in this manner it is recommended to run using some form
of persistent data volume. To do this we need to edit the `trow.yaml` file in
the `install` directory. The default setting is:

```
...
      volumes:
        - name: cert-vol
          emptyDir:
            medium: Memory
        - name: data-vol
          emptyDir: {}
```

We're only interested in the `data-vol` setting. Assuming your cluster supports
Kubernetes [Persistent
Volumes](https://kubernetes.io/docs/concepts/storage/persistent-volumes/), the
following should work:

```
...
      volumes:
        - name: cert-vol
          emptyDir:
            medium: Memory
        - name: data-vol
          persistentVolumeClaim:
            claimName: data-claim
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: data-claim
  namespace: kube-public
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```

This will request a 10GB volume on the cluster, that the Trow pod can read and
write to. A full example is found in [trow-gke.yaml](../install/trow-gke.yaml),
which has been tested on GKE.

The easiest way to use the new yaml file is to run the install script again.

If your cluster does not support Persistent Volumes, you may still be able to
use one of the other volume types that are [described in the
docs](https://kubernetes.io/docs/concepts/storage/volumes/#types-of-volumes).

Note that future versions of Trow are planned to run in a distributed HA manner,
which will reduce the liklihood of losing data through pod eviction when running
using the `emptyDir` volume type.

## Listing Repositories and Tags

Trow implements the [OCI Distribution
Specification](https://github.com/opencontainers/distribution-spec/blob/master/spec.md)
which includes API methods for listing repositories and tags. Unfortunately the
Docker CLI doesn't support these endpoints, so we need to use curl or a similar
tool. 

A full list of _repositories_ can be obtained by issuing a GET request to the
`/v2/_catalog` endpoint. For example:

```
curl --insecure https://trow.kube-public:31000/v2/_catalog
{"repositories":["repo","test","test/nginx","test/new","ng"]}
```

See below for instructions that avoid the use of `--insecure`.

For any given repository, we can use the `/v2/<repository>/tags/list` GET
endpoint to list the available tags e.g:

```
curl --insecure https://trow.kube-public:31000/v2/repo/tags/list
{"name":"repo","tags":["tag","tag3","tag2"]}
```

We can make the output a bit nicer by using `jq`:

```
$ curl --insecure -s https://trow.kube-public:31000/v2/repo/tags/list | jq
{
  "name": "repo",
  "tags": [
    "tag",
    "tag2",
    "tag3"
  ]
}
```

The catalog endpoint is a matter of debate by the OCI and may be replaced in
future versions.  Do not expect different registries to have compatible
implementations of this endpoint for historical reasons and ambiguities in
specification.

## Using Curl Securely

To avoid the need to use `--insecure` when talking to Trow, you need to provide
curl with the Certificate Authority certificate (_not_ the Trow cert, but the
authority that issued the cert). In a normal install, this will be the
Kubernetes CA. One way to do this is to pull it out of the service account secret:

```
$ kubectl get secret -o jsonpath="{.items[?(@.type==\"kubernetes.io/service-account-token\")].data['ca\.crt']}" | base64 --decode > k8sca.crt
```

Other methods are documented in this [Kubernetes ticket](https://github.com/kubernetes/kubernetes/issues/61572).

Once we have the certificate, we can use the `--cacert` argument with curl
instead of `--insecure`:

```
$ curl --cacert k8sca.crt https://trow.kube-public:31000/v2/
{}
```

## Troubleshooting

### Where are the logs?

The first place to look for debugging information is in the output from the
`kubectl describe` command. It's worth looking at the output for the deployment,
replicaset and pod:

```
$ kubectl describe deploy -n kube-public trow-deploy
$ kubectl describe replicaset -n kube-public trow-deploy
$ kubectl describe pod -n kube-public trow-deploy
```

In particular, look for problems pulling images or with containers crashing.

For the actual applcation logs try:

```
$ kubectl logs -n kube-public trow-deploy-596bf849c8-m7b7l
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
$ kubectl logs -n kube-public trow-deploy-596bf849c8-m7b7l -c trow-init
```

The `copy-certs` job may also log errors:

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

 - Verify that Trow is running (e.g. `kubectl get deploy -n kube-public
   trow-deploy`). If not, refer to the section on logs above to diagnose the
   issue. 
 - Check that the NodePort service exists (e.g. `kubectl describe svc -n
   kube-public trow`). 
 - Check that your network or cloud provider isn't blocking port 31000, if
   you're using GKE or AWS, you will likely need to configure networking rules.
 - Make sure that your client is pointing to the correct address. The IP address
   given in the error message should match the public IP of one of the cluster
   nodes. If it doesn't, try running the `install/configure-host.sh` script.

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

That means Trow considered the image name to refer to a _remote_ repository (i.e. not Trow itself) which has not been added to the allow list. If you believe the image should have been considered local, check the repository address appears in the list of addresses passed to Trow on start-up with the `-n` switch. If you want to allow a single remote image, add it to Trow by using the `--allow-images` flag. If you want to allow a whole repository or subdirectory of a repository use `--allow-prefixes`.

If the message reads:

```
Error creating: admission webhook "validator.trow.io" denied the request: Local image trow.kube-public:31000/notpresent disallowed as not contained in this registry and not in allow list
```

It means Trow expected to be able to serve this image itself but it wasn't found in the repository. Either push the image or use the `allow-images` or `allow-prefixes` flag to pre-approve images. Note that Kubernetes will keep trying to validate images.

If you get the error:

```
Error creating: Internal error occurred: failed calling admission webhook "validator.trow.io": Post https://trow.kube-public.svc:443/validate-image?timeout=30s: no endpoints available for service "trow"
```

Trow probably isn't running. You will need to disable the admission webhook and
restart Trow. To disable the webhook run `kubectl delete
validatingwebhookconfigurations.admissionregistration.k8s.io trow-validator`. If
Trow doesn't restart automatically, refer to the other sections on
troubleshooting or try reinstalling.

If the error is not to do with validation, it may be that the node is unable to
pull from the Trow registry. By default nodes are configured by the `copy-certs`
job. You can check that the job completed succesfully with `kubectl get jobs -n kube-public`. If the node is new, try running the script `install/copy-certs.sh`.


```
The push refers to repository [trow.kube-public:31000/test/nginx]
Get https://trow.kube-public:31000/v2/: x509: certificate signed by unknown authority
```

If you get this error, and you are using Docker for Mac, restart Docker.

