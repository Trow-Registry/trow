#!/bin/bash

set -euo pipefail
IFS=$'\n\t'

# Get service IP. Not sure how essential the IP addresses are, but let's do it
echo "Getting IP of trow service"
SERVICE_IP=$(dig +short trow.$POD_NAMESPACE.svc.cluster.local)
while [[ $SERVICE_IP == "" ]]
do
  sleep 2
  SERVICE_IP=$(dig +short trow.$POD_NAMESPACE.svc.cluster.local)
done

cat << EOF | cfssl genkey - | cfssljson -bare trow
{
  "hosts": [
    "trow.$POD_NAMESPACE.svc.cluster.local",
    "$POD_NAME.$POD_NAMESPACE.pod.cluster.local",
    "$POD_IP",
    "$SERVICE_IP"
  ],
  "CN": "trow.$POD_NAMESPACE.cluster.local",
  "key": {
    "algo": "rsa",
    "size": 2048
  }
}
EOF

# certs should be a volume that the main pod can read
cp trow-key.pem /certs/domain.key

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
    | base64 -d > /certs/ca.crt
