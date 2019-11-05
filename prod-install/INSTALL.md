Standard Kubernetes Install
===========================

At its heart Trow is just a single deploy that needs to be exposed through a service. The only minor
complication is getting TLS configured correctly. The developer or quick install uses a
cluster-signed cert that is used by Trow itself and copies the CA certificate to all nodes and
clients. The standard install runs Trow behind a TLS-terminating ingress. The TLS cert can be
obtained automatically via [cert-manager](https://github.com/jetstack/cert-manager) (or a
[ManagedCertificate](https://cloud.google.com/kubernetes-engine/docs/how-to/managed-certs) if
running on GKE), but does require that the client has a (sub)domain whose DNS can be pointed at
the cluster.

## Steps

 1) Update `cert.yaml` and `trow.yaml` with the domain name for your registry. Note that `cert.yaml`
 provisions a Google ManagedCertificate.
 2) Run `kubectl apply -k kustomize.yaml`
 3) Set the DNS for your domain to point to the IP for your ingress, which you can find with `kubectl
 get ingress -n trow`. Note that this IP is subject to change unless you obtain a static IP. It may
 take a moment for the IP address to populate.
 4) Once the certificate is obtained, TLS will start working and Trow should be available. You may
 get TLS errors whilst the certificate is being provisioned.

## Validation

To start validation, edit `validate.yaml` and set the URL to the domain where Trow is running. It
would be better to point Kubernetes at the internal Trow service, but as this isn't running over TLS
in this install, we need to use the external URL. We intend to address this in the future by running
Trow over TLS within the internal Kubernetes network.

## Troubleshooting

See the User Guide.

