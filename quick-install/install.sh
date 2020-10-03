#!/usr/bin/env bash
set -eo pipefail
unset CDPATH
IFS=$'\n\t'

cat << EOF
Trow AutoInstaller for Kubernetes
=================================

This installer assumes kubectl is configured to point to the cluster you want to
install Trow on and that your user has cluster-admin rights.

This installer will perform the following steps:

  - Create a ServiceAccount and associated Roles for Trow 
  - Create a Kubernetes Service and Deployment
  - Request and sign a TLS certificate for Trow from the cluster CA
  - Copy the public certificate to all nodes in the cluster
  - Copy the public certificate to this machine (optional)
  - Register a ValidatingAdmissionWebhook (optional) 

If you're running on GKE, you may first need to give your user cluster-admin
rights:

  $ kubectl create clusterrolebinding cluster-admin-binding --clusterrole=cluster-admin --user=\$(gcloud config get-value core/account)

Also make sure port 31000 is open on the firewall so clients can connect.
If you're running on the Google cloud, the following should work:

  $ gcloud compute firewall-rules create trow --allow tcp:31000 --project <project name>

EOF

runtime=$(kubectl get node -o=jsonpath='{.items[0].status.nodeInfo.containerRuntimeVersion}')
if [[ $runtime != docker* ]]
then
    echo "ERROR: Currently Docker is the only supported container runtime on nodes for the quick install."
    echo "Your cluster appears to be using $runtime."
    echo "Please refer to the standard install instructions to install Trow."
    exit 1
fi

namespace='kube-public'
if [ ! -z "$1" ]
then
	namespace=$1
fi


echo "This script will install Trow to the $namespace namespace."
#If default namespace, let them know how to change it
if [ -z "$1" ]
then
    echo "To choose a different namespace run:"
    echo "  $ $0 <my-namespace>"
fi
echo

while true
do
  read -r -p 'Do you want to continue? (y/n) ' choice
  case "$choice" in
    n|N) exit;;
    y|Y) break;;
    *) echo 'Response not valid';;
  esac
done

set +e
kubectl get ns $namespace &> /dev/null
if [[ $? != 0 ]]
then
    echo
    echo "The namespace $namespace doesn't exist."
    while true
    do
      read -r -p 'Should we create it now? (y/n) ' choice
      case "$choice" in
	n|N) exit;;
	y|Y) break;;
	*) echo 'Response not valid';;
      esac
    done
    set -e
    echo "Creating namespace $namespace"
    kubectl create ns $namespace
fi
set -e

on_mac=false
if [[ "$(uname -s)" = "Darwin" ]]; then
  on_mac=true
fi

namespace='kube-public'
if [ ! -z "$1" ]
then
	namespace=$1
fi

echo "Installing Trow in namespace: $namespace"
#change to directory with script so we can reach deps
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

echo
echo "Starting Kubernetes Resources"
sed "s/{{namespace}}/${namespace}/" trow.yaml | kubectl apply -f -

echo
echo "Approving certificate. This may take some time."
set +e
kubectl certificate approve "trow.${namespace}" &> /dev/null
rc=$?
while [[ $rc != 0 ]]
do
    sleep 1
    echo -n "."
    kubectl certificate approve "trow.${namespace}" &> /dev/null
    rc=$?
done
set -e

echo
echo "Saving cluster certficate as trow-ca-cert"
cert_file=$(mktemp /tmp/ca-cert.XXXXXX)
kubectl config view --raw --minify --flatten \
  -o jsonpath='{.clusters[].cluster.certificate-authority-data}' \
  | base64 --decode | tee -a $cert_file
kubectl create configmap trow-ca-cert --from-file=cert=$cert_file \
  --dry-run -o json | kubectl apply -n "$namespace" -f -

echo
./copy-certs.sh "$namespace"
echo

while true
do
  read -r -p 'Do you wish to install certs on this host and configure /etc/hosts to allow access from this machine? (y/n) ' choice
  case "$choice" in
    n|N) break;;
    y|Y) echo; ./configure-host.sh --namespace="$namespace" --add-hosts; break;;
    *) echo 'Response not valid';;
  esac
done

echo
while true
do
  read -r -p 'Do you want to configure Trow as a validation webhook (NB this will stop external images from being deployed to the cluster)? (y/n) ' choice
  case "$choice" in
    n|N) break;;
    y|Y) ./validate.sh "$namespace"; break;;
    *) echo 'Response not valid';;
  esac
done
