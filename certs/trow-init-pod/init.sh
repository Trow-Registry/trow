#!/bin/bash

set -euo pipefail
IFS=$'\n\t'

cfssl genkey req.json | cfssljson -bare trow
REQ=$(cat trow.csr | base64 | tr -d '\n')

kubectl delete csr trow.kube-public

cat <<EOF | kubectl create -f -
apiVersion: certificates.k8s.io/v1beta1
kind: CertificateSigningRequest
metadata:
  name: trow.kube-public
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

Please run "kubectl certificate approve trow.kube-public".
"""

STAT=$(kubectl get csr trow.kube-public -o json | jq -r .status.conditions[0].type)

while [[ $STAT != "Approved" ]]
do
  sleep 10
  STAT=$(kubectl get csr trow.kube-public -o json | jq -r .status.conditions[0].type)
done
