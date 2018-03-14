#!/bin/bash

# Temporary launch script to bootstrap certs that should go away

if [[ ! -f ./certs/domain.key && ! -f ./certs/ca.crt ]]; then
  cd certs
  ./make-certs.sh
  cd /
fi
exec "$@"

