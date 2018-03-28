#!/bin/bash

# Temporary launch script to bootstrap certs that should go away

if [[ ! -f ./certs/domain.key && ! -f ./certs/ca.crt ]]; then
  echo "No certs found, creating new ones"
  cd certs
  ./make-certs.sh
  cd /
fi
ls -l /certs
cat trow-default.toml

exec "$@"

