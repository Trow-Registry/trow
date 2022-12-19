#!/usr/bin/env bash
set -euo pipefail

#change to directory with script so we know where project root is
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

docker build \
    --platform linux/amd64 \
    --progress=plain \
    --add-host trow.test:127.0.0.1 \
    -t trow/test \
    -f Dockerfile.test \
    ..
