#!/usr/bin/env bash

set -euo pipefail

if [ ! -f conformance.test ]; then
    CONT=$(podman create ghcr.io/opencontainers/distribution-spec/conformance:v1.1.0)
    podman cp $CONT:/conformance.test .
    podman rm $CONT
fi

export OCI_ROOT_URL="http://127.0.0.1:8000"
export OCI_NAMESPACE="oci-conformance/distribution-test"

export OCI_TEST_PULL=1
export OCI_TEST_PUSH=1
export OCI_TEST_CONTENT_MANAGEMENT=1
export OCI_TEST_CONTENT_DISCOVERY=1
export OCI_TEST_CONTENT_MANAGEMENT=1
export OCI_HIDE_SKIPPED_WORKFLOWS=0

echo "Checking conformance against https://github.com/opencontainers/distribution-spec/blob/main/spec.md"
./conformance.test
