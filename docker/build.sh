#!/usr/bin/env bash
set -eo pipefail

# change to directory with script so we know where project root is
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

REPO="ghcr.io/extrality/trow-dev"
VERSION=$(sed '/^version = */!d; s///;q' ../Cargo.toml | sed s/\"//g)

TAG="$VERSION"
IMAGE="$REPO:$TAG"
DATE="$(date --rfc-3339=seconds)"

docker build \
  --build-arg VCS_REF="$(git rev-parse HEAD)" \
  --build-arg VCS_BRANCH="$(git symbolic-ref --short HEAD)" \
  --build-arg REPO="$REPO" \
  --build-arg TAG="$TAG" \
  --build-arg DATE="$DATE" \
  --build-arg VERSION="$VERSION" \
  -f Dockerfile \
  -t $IMAGE \
  ../
