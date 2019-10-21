# Trow User Guide

 * [Persisting Data/Images](#persisting-dataimages)
 * [Listing Repositories and Tags](#listing-repositories-and-tags)
 * [Using Curl Securely](#using-curl-securely)

More information is available in the [README](../README.md) and [Installation
instructions](../INSTALL.md).

## Persisting Data/Images

By default, Trow stores images and metadata in a Kubernetes [emptyDir
volume](https://kubernetes.io/docs/concepts/storage/volumes/#emptydir). This
means that the data will survive pod restarts, but will be lost if the Trow pod
is evicted from the node it is running on. This can occur when a node fails or
is brought down for maintenance.

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

