#!/usr/bin/env bash
set -exo pipefail

# change to directory with script so we know where project root is
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

GH_REPO="ghcr.io/trow-registry/trow"

# Check if cargo-sqlx is installed
if ! command -v cargo-sqlx >/dev/null 2>&1; then
    echo "→ sqlx-cli not found. Installing..."
    if ! cargo install sqlx-cli; then
        echo "Error: Failed to install sqlx-cli"
        exit 1
    fi
    echo "→ sqlx-cli installed successfully"
else
    echo "→ sqlx-cli is already installed"
fi

# Check if development database exists
DB_PATH="$src_dir/target/dev.db"
if [ ! -f "$DB_PATH" ]; then
    echo "→ Development database not found. Setting up..."
    cd "$src_dir/.."
    mkdir -p target
    if ! cargo sqlx database setup; then
        echo "Error: Failed to setup database"
        exit 1
    fi
    cd "$src_dir"
    echo "→ Database setup completed"
else
    echo "→ Development database already exists at $DB_PATH"
fi

# Use trow-multi builder if it exists, otherwise create it
if ! docker buildx ls | grep -s trow-multi ;
then
    # # Register binfmt handlers
    docker run --rm --privileged aptman/qus -s -- -p arm aarch64
    # Create new build instance
    docker buildx create --name trow-multi
fi
docker buildx use trow-multi

VERSION=$(sed '/^version = */!d; s///;q' ../Cargo.toml | sed s/\"//g)
TAG="$VERSION"
GH_IMAGE="$GH_REPO:$TAG"
DATE="$(date '+%Y-%m-%d %T%z')"

if [[ "$CI" = "true" || "$RELEASE" = "true" ]]
then
   PUSH="--push"
fi

echo $PUSH $GH_IMAGE $GH_REPO
# Can't load the image in the local registry... See https://github.com/docker/roadmap/issues/371
docker buildx build \
  --build-arg VCS_REF="${SOURCE_COMMIT:-$(git rev-parse HEAD)}" \
  --build-arg VCS_BRANCH="${SOURCE_BRANCH:-$(git symbolic-ref --short HEAD)}" \
  --build-arg REPO="$GH_REPO" \
  --build-arg TAG="$TAG" \
  --build-arg DATE="$DATE" \
  --build-arg VERSION="$VERSION" \
  $PUSH \
  --pull \
  --platform linux/arm64,linux/amd64 \
  -t $GH_IMAGE \
  -t $GH_REPO:default \
  -t $GH_REPO:latest \
  -f Dockerfile ../

if [[ "$CI" = true ]]
then
    echo "container-image=$GH_IMAGE" >> $GITHUB_OUTPUT
fi
