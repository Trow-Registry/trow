#!/bin/bash
set -euo pipefail

MAJOR_VERSION="0"
MINOR_VERSION="3"
PATCH_VERSION="0"

VERSION="$MAJOR_VERSION.$MINOR_VERSION.$PATCH_VERSION"

CARGO_VERSION=$(sed '/^version = */!d; s///;q' ../Cargo.toml | sed s/\"//g)

echo """
Release script for Trow. This script is dependent on docker and manifest-tool. 
You will also need to be logged into a Docker Hub account that can write to the containersol
repo.

Before running this, please check:
 - all dependencies have been updated (if not, do a PR)
 - all tests are passing
 - the version is correct in Cargo.toml and this file (currently $VERSION)
 - you are on the master branch and in sync with remote

Version in this script $VERSION
Version in Cargo.toml $CARGO_VERSION

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

./build.sh
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

# We now have the following images
docker push containersol/trow:$CARGO_VERSION-amd64
docker push containersol/trow:$CARGO_VERSION-armv7
docker push containersol/trow:$CARGO_VERSION-arm64

cp release-manifest.tmpl release-manifest.yaml
sed -i "s|{{FULL_VERSION}}|$VERSION|" ./release-manifest.yaml
sed -i "s|{{MAJOR_VERSION}}|$MAJOR_VERSION|" ./release-manifest.yaml
sed -i "s|{{MINOR_VERSION}}|$MAJOR_VERSION.$MINOR_VERSION|" ./release-manifest.yaml
sed -i "s|{{TROW_AMD64_IMAGE}}|containersol/trow:$CARGO_VERSION-amd64|" ./release-manifest.yaml
sed -i "s|{{TROW_ARMV7_IMAGE}}|containersol/trow:$CARGO_VERSION-armv7|" ./release-manifest.yaml
sed -i "s|{{TROW_ARM64_IMAGE}}|containersol/trow:$CARGO_VERSION-arm64|" ./release-manifest.yaml
manifest-tool push from-spec ./release-manifest.yaml

if [[ $(git rev-parse --abbrev-ref HEAD) != "master" ]]
then
    echo "Not on master branch. Refusing to tag."
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
