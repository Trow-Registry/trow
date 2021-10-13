Standard Kubernetes Install
===========================

At its heart, Trow is just a StatefulSet that needs to be exposed through a service. The main
complication is getting TLS configured correctly. The developer or quick install uses a
cluster-signed cert that is used by Trow itself and copies the CA certificate to all nodes and
clients. The standard install runs Trow behind a TLS-terminating ingress. The TLS cert can be
obtained automatically via [cert-manager](https://github.com/jetstack/cert-manager) (or a
[ManagedCertificate](https://cloud.google.com/kubernetes-engine/docs/how-to/managed-certs) if
running on GKE), but does require a (sub)domain whose DNS can be pointed at the cluster.

The following install instructions are based on Kustomize (see [Scott Lowe's
blog](https://blog.scottlowe.org/2019/09/13/an-introduction-to-kustomize/) for a good introduction).
If you'd rather try the new Helm install, please see [this guide](../docs/HELM_INSTALL.md).

To use the Kustomize install you will need to create a new overlay for your cluster, containing your
domain name and any special configuration (e.g. ingress). The overlay will refer to other
configuration files. By keeping all changes to your own directory, any updates to Trow base
configuration can be easily merged by just pulling the new commits. 

There is a complete example in the `install/overlays/example-overlay` directory, which can be used as a basis
for getting your cluster running. 

## Steps

Assuming your cluster has cert-manager installed:

 1) Copy the directory `install/overlays/example-overlay` to `install/overlays/mycluster`, changing mycluster to an
appropriate name.
 2) Open the `kustomization.yaml` file. Change the namespace to whatever namespace you want the Trow
resources to run in. Update the YAML under `secretGenerator` with the user name and password you 
want to use for the registry. 
 3) In the same `kustomization.yaml`, update the domain name to the domain you wish to use for your 
 registry. Update the user name to the user name you set in the previous step. 
 4) Run `kubectl apply -k overlays/mycluster` from the install directory.
 5) Set the DNS for your domain to point to the IP for your ingress, which you can find with `kubectl get ingress -n trow-example`. Note that in GKE, 
 this IP is subject to change unless you obtain a static IP.
 It may take a moment for the IP address to populate.
 6) Once the certificate is obtained, TLS will start working and Trow should be available. You may
 get TLS errors whilst the certificate is being provisioned.

If you're using a Google ManagedCertificate, change the base in `kustomization.yaml` to `../gke` and
replace the `Ingress` patch with a copy of `ManagedCertificate` patch from the
`install/overlays/gke/kustomization.yaml` directory and edit as appropriate.

For other installs, please use the provided files as a base and consider contributing new
overlays back to the project.

## Validation

To enable validation:

 1) Add the following to your `kustomization.yaml`

```
resources:
- ../../base/validate.yaml
```

And also uncomment the following under `patchesJson6902` and modify the domain value:

```yaml
    - patch: |-
        - op: replace
          path: /webhooks/0/name
          value: example.registry.com
      target:
        kind: ValidatingWebhookConfiguration
        name: trow-validator
        group: admissionregistration.k8s.io
        version: v1
```

 2) Run `kubectl apply -k overlays/mycluster` from the install directory.

It would be better to point Kubernetes at the internal Trow service in step 2, but as this isn't
running over TLS within the internal network in the default install, we need to use the external
URL. We intend to address this in a future release.

## Ingress Configuration.

Make sure your ingress is configured to allow large files to be transferred, or you may hit HTTP errors
such as `413 - Request Entity Too Large`. For the NGINX Ingress, make sure the following annotation is
present:

```
nginx.ingress.kubernetes.io/proxy-body-size: "0"
```

This will disable the transfer limit (it can alternatively be set to a large value such as `1000m`
to allow blobs up to 1000 MB in size). More details can be found in the [NGINX Ingress
guide](https://kubernetes.github.io/ingress-nginx/user-guide/nginx-configuration/annotations/#custom-max-body-size).
There is also an [Trow kustomize overlay with an NGINX
ingress](https://github.com/ContainerSolutions/trow/blob/main/install/overlays/cert-manager-nginx/ingress.yaml)
that may be useful.

## Full TLS

The default install will result in TLS being used between the registry client (Docker) and the
cluster ingress, but _not inside the cluster_. The next update to the Trow will use cluster
certificates to achieve full end-to-end TLS. 

NB if you use a service mesh, you may already have mutual TLS between pods and can safely ignore
this.

## Uninstall

This should be as simple as `kubectl delete -k overlays/mycluster`  (changing `overlays/mycluster`
to the name of your overlay).

## Troubleshooting

See the [User Guide](../docs/USER_GUIDE.md).

