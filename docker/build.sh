#!/bin/bash

#change to directory with script so we know where project root is
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

if [[ "$CI" = true ]]
then
    REPO=${DOCKER_REPO:-"docker.pkg.github.com/containersolutions/trow/trow"}
    VERSION=$(date +"%Y-%m-%d")-$GITHUB_RUN_ID
else
    REPO=${DOCKER_REPO:-"containersol/trow"}
    VERSION=$(sed '/^version = */!d; s///;q' ../Cargo.toml | sed s/\"//g)
fi
TAG=${DOCKER_TAG:-"default"}
IMAGE=${IMAGE_NAME:-"$REPO:$TAG"}
DATE="$(date --rfc-3339=seconds)"

docker build \
  --build-arg VCS_REF="${SOURCE_COMMIT:-$(git rev-parse HEAD)}" \
  --build-arg VCS_BRANCH="${SOURCE_BRANCH:-$(git symbolic-ref --short HEAD)}" \
  --build-arg REPO="$REPO" \
  --build-arg TAG="$TAG" \
  --build-arg DATE="$DATE" \
  --build-arg VERSION="$VERSION" \
  -f Dockerfile -t $IMAGE ../

if [[ "$CI" = true ]]
then
    docker push $IMAGE
fi
