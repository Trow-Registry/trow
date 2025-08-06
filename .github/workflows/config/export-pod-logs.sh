#!/usr/bin/env bash

mkdir -p ./pod_logs

for ns_pod in $(kubectl get pods -A -o go-template --template '{{range .items}}{{.metadata.namespace}}/{{.metadata.name}}{{"\n"}}{{end}}'); do
	read -r namespace pod <<<"$(echo "$ns_pod" | tr '/' ' ')"
	echo "Getting logs for $namespace $pod"
	kubectl logs --prefix=true --timestamps --all-containers=true -n "$namespace" "$pod" >"pod_logs/$namespace $pod.txt"
	echo "Done"
done

kubectl get events >pod_logs/k8s_events.txt
