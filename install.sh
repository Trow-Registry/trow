#!/bin/bash
set -eo pipefail
unset CDPATH
IFS=$'\n\t'

cat << EOF
Trow AutoInstaller for Kubernetes
=================================

This installer assumes kubectl is configued to point to the cluster you want to
install Trow on and that your user has cluster-admin rights.

This installer will perform the following steps:

  - Create a ServiceAccount and associated Roles for Trow 
  - Create a Kubernetes Service and Deployment
  - Request and sign a TLS certificate for Trow
  - Copy the public certificate to all nodes in the cluster
  - Copy the certificate to this machine (optional)
  - Register a ValidatingAdmissionWebhook (optional) 

If you're running on GKE, you may first need to give your user cluster-admin
rights:

  $ kubectl create clusterrolebinding cluster-admin-binding --clusterrole=cluster-admin --user=<user>

Where <user> is your user, normally the e-mail address you use with your GKE 
account.

EOF

while true
do
  read -r -p 'Do you want to continue? (y/n) ' choice
  case "$choice" in
    n|N) exit;;
    y|Y) break;;
    *) echo 'Response not valid';;
  esac
done

on_mac=false
if [[ "$(uname -s)" = "Darwin" ]]; then
  on_mac=true
fi

#change to directory with script so we can reach deps
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

echo "Starting Kubernetes Resources"
kubectl apply -f install/trow.yaml

echo "Approving certificate. This may take some time."
set +e
kubectl certificate approve trow.kube-public &> /dev/null
rc=$?
while [[ $rc != 0 ]]
do
    sleep 1
    echo -n "."
    kubectl certificate approve trow.kube-public &> /dev/null
    rc=$?
done
set -e

cd install
./copy-certs.sh

while true
do
  read -r -p 'Do you wish to install certs on this host and configure /etc/hosts to allow access from this machine? (y/n) ' choice
  case "$choice" in
    n|N) break;;
    y|Y) ./configure-host.sh --add-hosts; break;;
    *) echo 'Response not valid';;
  esac
done

while true
do
  read -r -p 'Configure validation webhook (NB this will stop external images from being deployed to the cluster)? (y/n) ' choice
  case "$choice" in
    n|N) break;;
    y|Y) ./validate.sh; break;;
    *) echo 'Response not valid';;
  esac
done
