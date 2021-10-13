# Helm Install Trow

Use the following instructions to install Trow via [Helm](https://helm.sh). You may also find the
information in the [standard install instructions](../install/INSTALL.md) useful.

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

## Configuration

| Parameter                  | Description                                                                                       | Default                                                                    |
|----------------------------|---------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------|
| trow.domain                | The Domain that Trow will be served on, you will need to setup the DNS to point to the correct IP | myregistry.mydomain.io                                                     |
| trow.user                  | admin user name                                                                                   | user                                                                       |
| trow.password              | admin password                                                                                    | password                                                                   |
| trow.validatingWebhooks.enabled  | enable the validation webhooks that block unauthorized images                                     | false                                                                      |
| imagePullSecrets           | secret used to pull the image (not needed if using the default image)                             | []                                                                         |
| service.type               | type on the service ( ClusterIP, NodePort, LoadBalancer)                                          | NodePort                                                                   |
| service.port               | Port to expose the service on                                                                     | 8000                                                                       |
| ingress.enabled            | Enable the ingress setup                                                                          | false                                                                      |
| ingress.annotations        | List of annotations to set on the ingress                                                         | {}                                                                         |
| ingress.hosts              | Host configuration for the ingress                                                                | [{host: null, paths: ['/']}}                                               |
| ingress.gke                | Set to true if you are using GKE's managed SSL certificates                                       | false                                                                      |
| ingress.tls                | TLS configuration for the Ingress                                                                 | []                                                                         |
| resources                  | Resource Limits and quotas (currently no limits or requests set)                                  | {}                                                                         |
| nodeSelector               | Selector to define which nodes to put the pods on                                                 | {}                                                                         |
| tolerations                | Any toleration values to be set on the pods                                                       | []                                                                         |
| affinity                   | Any affinity rules to be set on the pod                                                           | {}                                                                         |
| volumeClaim                | As trow uses a statefulset and uses a volume to store data this can be configured accordingly     | {accessModes: ["ReadWriteOnce"], resources: {requests: {storage: "20Gi"}}} |
| replicaCount               | Amount of replicas of trow to run                                                                 | 1                                                                          |
