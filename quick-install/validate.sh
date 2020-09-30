#!/usr/bin/env bash
set -eo pipefail
unset CDPATH
IFS=$'\n\t'

namespace='kube-public'
if [ ! -z "$1" ]
then
	namespace=$1
fi

echo "Setting up trow as a validating webhook"
echo "WARNING: This will limit what images can run in your cluster"
echo "By default, only images in Trow and official Kubernetes images will be 
allowed"
echo

cabundle=$(kubectl get configmap trow-ca-cert -n "$namespace" -o jsonpath='{.data.cert}' | openssl base64 | tr -d '\n')
#Really not happy about use of sed here
#Can we use go-template now?
sed "s/{{cabundle}}/${cabundle}/" validate-tmpl.yaml | sed "s/{{namespace}}/${namespace}/" > validate.yaml
kubectl apply -f validate.yaml
