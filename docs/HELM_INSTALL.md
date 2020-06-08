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
Trow needs to be served Via `TLS`, by default the ingress is disabled and the pod runs without TLS, so you would need to expose the trow pod (the prefered method is via ingress) and add TLS to this, the default way using:
### GKE
create a managed certificate as followed:
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

| Parameter                  | Description |                                                                            |
|----------------------------|-------------|----------------------------------------------------------------------------|
| trow.domain                |             | myregistry.mydomain.io                                                     |
| trow.user                  |             | user                                                                       |
| trow.password              |             | password                                                                   |
| trow.webhooks.enabled      |             | false                                                                      |
| imagePullSecrets           |             | []                                                                         |
| service.type               |             | NodePort                                                                   |
| service.port               |             | 8000                                                                       |
| ingress.enabled            |             | false                                                                      |
| ingress.annotations        |             | {}                                                                         |
| ingress.hosts              |             | [{host: null, paths: ['/']}}                                               |
| ingress.tls                |             | []                                                                         |
| resources                  |             | {}                                                                         |
| nodeSelector               |             | {}                                                                         |
| tolerations                |             | []                                                                         |
| affinity                   |             | {}                                                                         |
| volumeClaim                |             | {accessModes: ["ReadWriteOnce"], resources: {requests: {storage: "20Gi"}}} |
| replicaCount               |             | 1                                                                          |
| securityContext.fsGroup    |             | 999                                                                        |
| securityContext.runAsGroup |             | 999                                                                        |
| securityContext.runAsUser  |             | 999                                                                        |
