# Trow User Guide

More information is available in the [README](../README.md) and [Installation
instructions](../INSTALL.md.

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
curl --cacert cert2.pem https://trow.kube-public:31000/v2/repo/tags/list
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

To avoid the need to use `--insecure` when talking to Trow, you first need to
get the Certificate Authority certificate (_not_ the Trow cert, but the
authority that issued the cert). In a normal install, this will be the
Kubernetes CA. One way to do this is via a service account secret:

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

