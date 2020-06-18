# Helm Install Trow

## Add the Trow Helm Repo

```bash
helm repo add trow https://trow.io
```

## Install Trow with Helm

```bash
helm install trow trow/trow
```
## Notes on installation
Trow needs to be served Via `TLS`, by default the ingress is disabled, so you would need to expose the trow pod (the prefered method is via ingress) and add TLS to this, the default way using:
### Google Managed Certificates on GKE
create a managed certificate as follows:
```yaml
apiVersion: networking.gke.io/v1beta1
kind: ManagedCertificate
metadata:
  name: trow-certificate
spec:
  domains:
    - myregistry.mydomain.com
```

and annotate the ingress using the value `ingress.annotations`:
```yaml
# values.yaml
ingress:
    enabled: true
    gke: true
    annotations: 
        networking.gke.io/managed-certificates: trow
```
 then install specifying the `values.yaml` from above
```bash
helm install \
    -f values.yaml  \
    trow \
    trow/trow
```

### Cert-Manager
if you have `cert manager` installed you can use the following values:
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
