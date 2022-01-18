#!/usr/bin/env bash
set -euo pipefail

MAJOR_VERSION="0"
MINOR_VERSION="3"
PATCH_VERSION="5"
# Only use this for "special" release and prefix with "-" 
# e.g. -SCANNING for scanning preview feature release
NAME="" 

VERSION="$MAJOR_VERSION.$MINOR_VERSION.$PATCH_VERSION$NAME"

CARGO_VERSION=$(sed '/^version = */!d; s///;q' ../Cargo.toml | sed s/\"//g)
SERVER_CARGO_VERSION=$(sed '/^version = */!d; s///;q' ../lib/server/Cargo.toml | sed s/\"//g)
BRANCH=$(git branch --show-current)

echo """
Release script for Trow. This script is dependent on docker and manifest-tool. 
You will also need to be logged into a Docker Hub account that can write to the containersol
repo.

Before running this, please check:
 - all dependencies have been updated (if not, do a PR)
 - all tests are passing
 - the version is correct in Cargo.toml and this file (currently $VERSION)
 - you are on the main branch and in sync with remote
 - you can push to the containersol Hub repo

Version in this script $VERSION
Version in Cargo.toml $CARGO_VERSION
Version in lib/server/Cargo.toml $SERVER_CARGO_VERSION
You are currently on $BRANCH

"""

while true
do
  read -r -p "About to build Docker images. This will take a while. Continue(y/n)? " choice
  case "$choice" in
    n|N) exit;;
    y|Y) break;;
    *) echo 'Response not valid';;
  esac
done

./multi-arch.sh

while true
do
  read -r -p "About to push containersol/trow:$VERSION. Continue(y/n)? " choice
  case "$choice" in
    n|N) exit;;
    y|Y) break;;
    *) echo 'Response not valid';;
  esac
done

docker tag containersol/trow:$VERSION ghcr.io/containersolutions/trow/trow:$VERSION
docker push containersol/trow:$VERSION
docker push ghcr.io/containersolutions/trow/trow:$VERSION

if [[ $(git rev-parse --abbrev-ref HEAD) != "main" ]]
then
    echo "Not on main branch. Refusing to tag."
    exit 1
fi

while true
do
  read -r -p "Tagging git as "v$VERSION". Continue(y/n)? " choice
  case "$choice" in
    n|N) exit;;
    y|Y) break;;
    *) echo 'Response not valid';;
  esac
done
git tag v$VERSION
git push origin v$VERSION

# Next do the Helm Release
# Update charts/trow/Chart.yaml with correct values
# Run `helm package charts/trow/ --destination charts/` - this should build tgz
# Upload tgz to GH release
# Run `helm repo index charts/ --merge charts/index.yaml --url https://github.com/ContainerSolutions/trow/releases/download/v0.3.4/` 
# Merge new index.yaml
# Copy index.yaml to gh-pages branch
