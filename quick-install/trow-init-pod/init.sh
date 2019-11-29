#!/bin/bash

IFS=$'\n\t'

date
# Only do this if we don't already have a cert
# There is a danger the cert is incorrect or expired
# Need to add some checks

if [[ $(kubectl get secret trow-tls) ]]; then

  echo "Found existing trow-tls certificate"
  # save it out to tmpfs volume at /certs/

  kubectl get secret -n $POD_NAMESPACE trow-tls -o jsonpath="{.data.tls\.crt}" \
    | base64 --decode > /certs/domain.crt
  kubectl get secret -n $POD_NAMESPACE trow-tls -o jsonpath="{.data.tls\.key}" \
    | base64 --decode > /certs/domain.key

else
  set -euo pipefail
  echo "Generating new certificate"
  # Get service IP. Not sure how essential the IP addresses are, but let's do it
  echo "Getting IP of trow service"
  SERVICE_IP=$(dig +short +search trow.$POD_NAMESPACE)
  while [[ $SERVICE_IP == "" ]]
  do
    sleep 2
    SERVICE_IP=$(dig +short +search trow.$POD_NAMESPACE)
  done

  echo "POD NAMESPACE: $POD_NAMESPACE"
  echo "POD NAME: $POD_NAME"
  echo "POD IP: $POD_IP"
  echo "SERVICE IP: $SERVICE_IP"

  cd /tmp
  cat << EOF | cfssl genkey - | cfssljson -bare trow
{
  "hosts": [
    "trow.$POD_NAMESPACE.svc.cluster.local",
    "trow.$POD_NAMESPACE.svc",
    "$POD_NAME.$POD_NAMESPACE.pod.cluster.local",
    "trow.$POD_NAMESPACE.svc",
    "trow.$POD_NAMESPACE",
    "$POD_IP",
    "$SERVICE_IP"
  ],
  "CN": "trow.$POD_NAMESPACE",
  "key": {
    "algo": "rsa",
    "size": 4096
  }
}
EOF

  # Key is now saved to /tmp/trow-key.pem
  mv /tmp/trow-key.pem /certs/domain.key
  # TODO: pipe straight to tmpfs volume

  REQ=$(cat trow.csr | base64 | tr -d '\n')

  # Change to output warning and exit instead.
  # Can't reuse CSRs due to changing IPs.
  # Can Docker clients trust k8s CA rather than certs?
  kubectl delete csr trow.$POD_NAMESPACE || true

  cat <<EOF | kubectl create -f -
apiVersion: certificates.k8s.io/v1beta1
kind: CertificateSigningRequest
metadata:
  name: trow.$POD_NAMESPACE
spec:
  groups:
  - system:authenticated
  request: $REQ
  usages:
  - digital signature
  - key encipherment
  - server auth
EOF

  echo """
Waiting for CSR to be approved.

Please run:

$ kubectl certificate approve trow.$POD_NAMESPACE
"""

  STAT=$(kubectl get csr trow.$POD_NAMESPACE -o json | jq -r .status.conditions[0].type)

  while [[ $STAT != "Approved" ]]
  do
    sleep 10
    STAT=$(kubectl get csr trow.$POD_NAMESPACE -o json | jq -r .status.conditions[0].type)
  done

  kubectl get csr trow.$POD_NAMESPACE -o jsonpath='{.status.certificate}' \
      | base64 -d > /certs/domain.crt

  kubectl create secret tls trow-tls -n $POD_NAMESPACE --key="/certs/domain.key" --cert="/certs/domain.crt"

  echo
  echo "Saved certificate and key to trow-tls secret"
  echo
fi

echo "Init completed succesfully"
