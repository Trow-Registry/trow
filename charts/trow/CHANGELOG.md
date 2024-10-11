# Changelog

## unreleased

* Templates:
  * webhooks: switch from daemonset to deployment (+ PDB to ensure at least 1 pod is always running)
  * rename `enable` to `enabled`
* `Values.yaml`:
  * `.webhookPatch` renamed `.webhooks.tls.patch`
  * add in `.webhooks.tls`: `certmanager` & `existingSecretRef`

## v0.7.0

* Renamings in `Values.yaml`:
  * `trow.validation` -> `trow.validationWebhook`
  * `trow.proxyConfig` -> `trow.proxyRegistries`
    * `proxyConfig.enableWebhook` -> `proxyRegistries.webhook.enable`
  * `onWebhookFailure` -> `failurePolicy`

**Full Changelog**: <https://github.com/Trow-Registry/trow/compare/trow-0.7.0...trow-0.6.4>