#!/usr/bin/env bash

mkdir -p ./pod_logs

for pod in $(kubectl get pods -A -o go-template --template '{{range .items}}{{.metadata.namespace}}/{{.metadata.name}}{{"\n"}}{{end}}')
do
    unpacked_pod=$(echo $pod | tr '/' ' ')
    echo "Getting logs for $unpacked_pod"
    kubectl logs -n $unpacked_pod > "pod_logs/$unpacked_pod".txt
    echo "Done"
done
