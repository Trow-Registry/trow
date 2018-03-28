#!/bin/bash
set -e
set -o pipefail

registry_host="trow.kube-public"
registry_port="31000"
registry_host_port="${registry_host}:${registry_port}"

mkdir --parents "/etc/docker/certs.d/$registry_host_port/"
echo "copying certs"
kubectl get configmap trow-cert -n kube-public -o jsonpath='{.data.cert}'
    > "/etc/docker/certs.d/$registry_host_port/ca.crt"
echo "Successfully copied certs"
