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

echo "Adding entry to /etc/hosts"
# sed would be a better choice than ed, but it wants to create a temp file :(
printf "g/$registry_host/d\nw\n" | ed /hostfile

# We could use the service IP for trow, but this may not be accessible by the
# host, so just use NodePort on our host
echo "127.0.0.1 $registry_host # Added by trow install" >> /hostfile
echo "Added entry"

