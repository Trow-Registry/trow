#!/usr/bin/env bash
set -eo pipefail

#change to directory with script so we know where project root is
#https://stackoverflow.com/questions/59895/can-a-bash-script-tell-which-directory-it-is-stored-in
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

GH_REPO=${DOCKER_REPO:-"ghcr.io/extrality/trow/trow"}
DH_REPO=${DOCKER_REPO:-"containersol/trow"}

# Use trow-multi builder if it exists, otherwise create it
set +e
if ! docker buildx ls | grep -s trow-multi ;
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
DH_IMAGE=${IMAGE_NAME:-"$DH_REPO:$TAG"}
GH_IMAGE=${IMAGE_NAME:-"$GH_REPO:$TAG"}
DATE="$(date '+%Y-%m-%d %T%z')"

if [[ "$CI" = "true" || "$RELEASE" = "true" ]]
then
   PUSH="--push"
fi

echo $PUSH $DH_IMAGE $GH_IMAGE $DH_REPO $GH_REPO
docker buildx build \
  --build-arg VCS_REF="${SOURCE_COMMIT:-$(git rev-parse HEAD)}" \
  --build-arg VCS_BRANCH="${SOURCE_BRANCH:-$(git symbolic-ref --short HEAD)}" \
  --build-arg REPO="$DH_REPO" \
  --build-arg TAG="$TAG" \
  --build-arg DATE="$DATE" \
  --build-arg VERSION="$VERSION" \
  $PUSH --pull --platform linux/arm64,linux/amd64 \
  -t $DH_IMAGE -t $GH_IMAGE -t $DH_REPO:default -t $GH_REPO:default \
  -t $DH_REPO:latest -t $GH_REPO:latest \
  -f Dockerfile ../

# Sign the images
# Assumes runner has installed cosing e.g. uses: sigstore/cosign-installer@main
if [[ "$CI" = true ]]
then
    #sign once for each registry, will sign corresponding hash
    #(assumes keyless signing is enabled)
    cosign sign --recursive $DH_IMAGE
    cosign sign --recursive $GH_IMAGE
fi
