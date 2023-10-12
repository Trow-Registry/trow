# Helm Install Trow

Use the following instructions to install Trow via [Helm](https://helm.sh).

## Add the Trow Helm Repo

```bash
helm repo add trow https://trow.io
```

## Install Trow with Helm

```bash
helm install trow trow/trow
```
## Notes on installation

The Docker client expects to use TLS to push and pull images from registries.
This is normally accomplished by exposing Trow via a Kubernetes ingress endpoint with TLS
configured. How to do this is dependent on the Kubernetes cluster configuration, but the following
gives examples for GKE managed certificates and cert-manager. Note that you will need a (sub)domain
name for Trow.

### Google Managed Certificates on GKE

First, create a managed certificate for your domain. You can use the following YAML, replacing
the domain name as appropriate:

```yaml
apiVersion: networking.gke.io/v1beta1
kind: ManagedCertificate
metadata:
  name: trow-certificate
spec:
  domains:
    - myregistry.mydomain.com
```

Apply the YAML as normal e.g. `kubectl apply -f cert.yaml`.

We then need to update the ingress settings in `values.yaml` to specify we are using GKE and add
an annotation with the name of the certificate e.g:

```yaml
# values.yaml
ingress:
    enabled: true
    gke: true
    annotations:
        networking.gke.io/managed-certificates: trow-certificate
```

Finally we can install Trow as normal, using the updated `values.yaml`:

```bash
helm install \
    -f values.yaml  \
    trow \
    trow/trow
```

### Cert-Manager

If you have `cert manager` installed you can use the following values:
```yaml
# values.yaml
ingress:
    enabled: true
    annotations:
        cert-manager.io/cluster-issuer: nameOfClusterIssuer
    tls:
    - hosts:
      - myregistry.mydomain.io
      secretName: myingress-cert
```

then install specifying the `values.yaml` from above
```bash
helm install \
    -f values.yaml  \
    trow \
    trow/trow
```
