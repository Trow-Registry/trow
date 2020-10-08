#!/usr/bin/env bash

#change to directory with script so we know where project root is
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"
docker build --add-host trow.test:127.0.0.1 -t test -f Dockerfile.test ..
