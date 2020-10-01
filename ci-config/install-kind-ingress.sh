#!/bin/bash
set -e

# Apply the ingress
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/master/deploy/static/provider/kind/deploy.yaml

# Script to wait for ingress to be up and running
# TODO: Add timeout
ingress_running=false
while [[ "$ingress_running" != "true" ]]
do
    status=$(kubectl -n ingress-nginx get deployment ingress-nginx-controller -o jsonpath="{.status.availableReplicas}")
    if [[ "$status" = "1" ]]; then
        ingress_running=true
    fi
done
echo "Ingress Running"
