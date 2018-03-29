#!/bin/bash
set -eo pipefail
unset CDPATH

registry_host="trow.kube-public"
registry_port="31000"
registry_host_port="${registry_host}:${registry_port}"
add_host_ip=""

function get_ip_from_k8s {

  add_host_ip=""
  local schedulable_nodes=""
  schedulable_nodes="$(kubectl get nodes -o template \
    --template='{{range.items}}{{if not .spec.unschedulable}}{{range.status.addresses}}{{if eq .type "ExternalIP"}}{{.address}} {{end}}{{end}}{{end}}{{end}}')" 

  for n in $schedulable_nodes 
  do
    add_host_ip=$n
    break
  done

  if [[ -z "$add_host_ip" ]]; then
    echo
    echo 'No external IP listed in "kubectl get nodes -o wide"'
    echo "Trying minikube"

    set +e
    add_host_ip=$(minikube ip)
    set -e
    if [[ -z "$add_host_ip" ]]; then
      echo "Failed to discover IP for kubernetes node"
      exit 2
    fi
  fi
}

function add_to_etc_hosts { 
  echo
  echo "Exposing registry via /etc/hosts"

  # sed would be a better choice than ed, but it wants to create a temp file :(
  # redirectred stderr here, as ed likes to write to it even in success case

  if [[ -n "$add_host_ip" ]]; then
    printf "g/%s/d\nw\n" "$add_host_name" \
      | ed /etc/hosts 2> /dev/null

    echo "$add_host_ip $add_host_name # added for trow registry" >> /etc/hosts
  else
    echo
    echo "Failed to find external address for cluster" >&2
    exit 2
  fi
  echo 
  echo "Successfully configured localhost"
  return 0
}
########
# MAIN #
########

#Copy cert to host

if [[ $(id -u) -ne 0 ]]; then
  echo "Installing certificate requires root privileges i.e:"
  echo 
  echo "$ sudo $0 $@"
  exit 1
fi

mkdir --parents "/etc/docker/certs.d/$registry_host_port/"
echo "Copying cert into Docker"

kubectl get configmap trow-cert -n kube-public -o jsonpath='{.data.cert}' \
    > "/etc/docker/certs.d/$registry_host_port/ca.crt"

echo "Successfully copied cert"

if [[ "$1" = "--add-hosts" ]]; then
  echo "Adding entry to /etc/hosts for trow.kube-public" 
  get_ip_from_k8s
  add_host_name="trow.kube-public"
  add_to_etc_hosts
fi
