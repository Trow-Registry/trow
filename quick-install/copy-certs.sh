#!/bin/bash
set -eo pipefail
unset CDPATH
IFS=$'\n\t'

namespace='kube-public'
if [ ! -z "$1" ]
then
	namespace=$1
fi

echo
echo "Copying certs to nodes"

#delete any old jobs
for job in $(kubectl get jobs -n $namespace -o go-template --template '{{range .items}}{{.metadata.name}}

{{end}}') # blank line is important
do
  if [[ $job = copy-certs* ]]; then
    kubectl delete -n $namespace job "$job"
  fi
done
tmp_file=$(mktemp)
kubectl get nodes -o go-template-file --template copy-certs-tmpl.yaml > "$tmp_file"
kubectl create -f "$tmp_file"
rm "$tmp_file"

