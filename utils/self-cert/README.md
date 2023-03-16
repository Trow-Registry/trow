# Running Trow with Self-signed Cert

The default way to run Trow is currently inside Kubernetes using a cert signed
by the Kubernetes CA.

If you'd rather use a self-signed cert, whether to run outside of Kubernetes or
just to avoid CSRs, feel free to use the templates in this folder. All they do
is run openssl to create an RSA cert (don't use ECDSA as it's not supported
currently).
