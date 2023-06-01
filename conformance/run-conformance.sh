#!/usr/bin/env bash

# This script will build and run the Trow registry, then build and run
# the OCI conformance tests against it.
# It depends on bash, docker and git being available.

set -eu

spec_name=distribution-spec
spec_version=v1.0
prod_name=example
trow_version=v0.2.0

# check out trow repo
rm -rf trow-tmp && git clone --branch "$trow_version" git://github.com/extrality/trow.git trow-tmp
(cd trow-tmp/docker && ./build.sh) # Builds containersol/trow:default image
rm -rf trow-tmp
# start trow
docker network create trow-conf-test || true
docker stop trow || true
docker rm trow || true
docker run --net trow-conf-test --name trow -d containersol/trow:default

# check out conformance repo
rm -rf conf-tmp && git clone https://github.com/opencontainers/${spec_name}.git conf-tmp
(cd conf-tmp && docker build -t conformance:latest -f Dockerfile .)
rm -rf conf-tmp
# Would delete results as well, but they're owned by root :(
docker run --rm \
  --net trow-conf-test \
  -v $(pwd)/results:/results \
  -w /results \
  -e OCI_ROOT_URL="http://trow:8000" \
  -e OCI_NAMESPACE="myorg/myrepo" \
  -e OCI_DEBUG="true" \
  conformance:latest
cp results/report.html ./
cp results/junit.xml ./
docker stop trow && docker rm trow
docker network rm trow-conf-test
