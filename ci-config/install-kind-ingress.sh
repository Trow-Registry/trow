#!/bin/bash
set -eo pipefail

# Apply the ingress
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/master/deploy/static/provider/kind/deploy.yaml

# Script to wait for ingress to be up and running
ingress_running=false
timeout = 300 # seconds
while [[ "$ingress_running" != "true" -a $timeout -gt 0 ]]
do
    status=$(kubectl -n ingress-nginx get deployment ingress-nginx-controller -o jsonpath="{.status.availableReplicas}")
    if [[ "$status" = "1" ]]; then
        ingress_running=true
    fi
    sleep 1
    timeout--
done
echo "Ingress Running"
