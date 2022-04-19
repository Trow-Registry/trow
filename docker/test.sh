#!/usr/bin/env bash
set -euo pipefail

#change to directory with script so we know where project root is
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

export DOCKER_BUILDKIT=1
docker build \
    --progress=plain \
    --build-arg BUILDKIT_SANDBOX_HOSTNAME=trow.test \
    -t trow/test \
    -f Dockerfile.test \
    ..
