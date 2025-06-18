# Changelog

## v0.11.0 (2025-06-18)

* Trow v0.8.0
* Add config hash to pod labels
* Add pod and container security context
* Change uid and gid to be in a sane range

## v0.10.2 (2025-03-05)

* Trow v0.7.4

## v0.10.1 (2025-02-24)

* Trow v0.7.3 (fix broken arm64 image)

## v0.10.0 (2025-02-21)

* Trow v0.7.2
* Add option to limit disk space usage

## v0.9.0 - v0.9.1 (2025-01-28)

* The chart is now pulled via OCI (`oci://ghcr.io/trow-registry/charts/trow`)
* Upgrade trow to `v0.7.0`
* Use strict helm hooks ordering
* Support for ingressClassName
* Fix namespaceSelector

## v0.8.1

Fix `certmanager.k8s.io/inject-ca-from` annotation not correctly referencing the `Certificate`.

## v0.8.0

* Templates:
  * webhooks: switch from daemonset to deployment (+ PDB to ensure at least 1 pod is always running)
  * rename `enable` to `enabled`
* `Values.yaml`:
  * `.webhookPatch` renamed `.webhooks.tls.patch`
  * add in `.webhooks.tls`: `certmanager` & `existingSecretRef`
  * add `namespaceSelector` to `.webhooks`

## v0.7.0

* Renamings in `Values.yaml`:
  * `trow.validation` -> `trow.validationWebhook`
  * `trow.proxyConfig` -> `trow.proxyRegistries`
    * `proxyConfig.enableWebhook` -> `proxyRegistries.webhook.enable`
  * `onWebhookFailure` -> `failurePolicy`

**Full Changelog**: <https://github.com/Trow-Registry/trow/compare/trow-0.7.0...trow-0.6.4>
