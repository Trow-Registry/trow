#!/bin/bash

#change to directory with script so we know where project root is
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"
docker build -f Dockerfile -t containersol/trow:default ../
