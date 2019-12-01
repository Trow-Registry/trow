#!/bin/bash
set -eo pipefail
unset CDPATH
IFS=$'\n\t'

echo
echo "Copying certs to nodes"

NAMESPACE=kube-public
if [ -n $1 ]
then
	NAMESPACE=$1

#delete any old jobs
for job in $(kubectl get jobs -n $NAMESPACE -o go-template --template '{{range .items}}{{.metadata.name}}

{{end}}') # blank line is important
do
  if [[ $job = copy-certs* ]]; then
    kubectl delete -n $NAMESPACE job "$job"
  fi
done
tmp_file=$(mktemp)
kubectl get nodes -o go-template-file --template copy-certs-tmpl.yaml > "$tmp_file"
kubectl create -f "$tmp_file" -n $NAMESPACE
rm "$tmp_file"

