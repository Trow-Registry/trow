#!/usr/bin/env bash
set -eo pipefail

#change to directory with script so we know where project root is
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

GH_REPO=${DOCKER_REPO:-"ghcr.io/containersolutions/trow/trow"}
REPO=${DOCKER_REPO:-"containersol/trow"}

# Use trow-multi builder if it exists, otherwise create it
set +e
docker buildx ls | grep -s trow-multi
if [[ $? != 0 ]]
then
    # Register binfmt handlers
    docker run --rm --privileged aptman/qus -s -- -p arm aarch64
    # Create new build instance
    docker buildx create --name trow-multi
fi
set -e
docker buildx use trow-multi

# If we're in a github action, set the image name differently
if [[ "$CI" = true ]]
then
    VERSION=$(date +"%Y-%m-%d")-$GITHUB_RUN_NUMBER
else
    VERSION=$(sed '/^version = */!d; s///;q' ../Cargo.toml | sed s/\"//g)
fi

TAG=${DOCKER_TAG:-"$VERSION"}
IMAGE=${IMAGE_NAME:-"$REPO:$TAG"}
DATE="$(date --rfc-3339=seconds)"

if [[ "$CI" = true ]]
then
   PUSH = "--push"
fi

docker buildx build \
  --build-arg VCS_REF="${SOURCE_COMMIT:-$(git rev-parse HEAD)}" \
  --build-arg VCS_BRANCH="${SOURCE_BRANCH:-$(git symbolic-ref --short HEAD)}" \
  --build-arg REPO="$REPO" \
  --build-arg TAG="$TAG" \
  --build-arg DATE="$DATE" \
  --build-arg VERSION="$VERSION" \
  $PUSH --pull --platform linux/arm/v7,linux/arm64,linux/amd64 \
  -f "Dockerfile" -t $IMAGE ../

docker tag $IMAGE containersol/trow:default
docker tag $IMAGE $GH_REPO:default
docker tag $IMAGE containersol/trow:latest
docker tag $IMAGE $GH_REPO:latest
docker push containersol/trow:default $GH_REPO:default containersol/trow:latest $GH_REPO:latest
