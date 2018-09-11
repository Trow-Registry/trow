#!/bin/bash

# Temporary launch script to bootstrap certs that should go away

if [[ $@ != *"--no-tls"* && ! -f ./certs/domain.key && ! -f ./certs/ca.crt ]]; then
  echo "No certs found, creating new ones"
  mkdir ./certs || true
  cp install/self-cert/* ./certs/
  cd certs
  ./make-certs.sh
  cd /
fi
echo "Running /trow $@"

exec env RUST_LOG="warn" /trow "$@"

