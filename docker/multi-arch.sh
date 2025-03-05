#!/usr/bin/env bash
set -exo pipefail

# change to directory with script so we know where project root is
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

docker="$(command -v docker 2> /dev/null || echo "podman")"
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

if [ "$docker" = "docker" ]; then
    # Use trow-multi builder if it exists, otherwise create it
    # Not needed for podman
    if ! docker buildx ls | grep -s trow-multi ;
    then
        # # Register binfmt handlers
        docker run --rm --privileged aptman/qus -s -- -p arm aarch64
        # Create new build instance
        docker buildx create --name trow-multi
    fi
    docker buildx use trow-multi
fi

VERSION=$(sed '/^version = */!d; s///;q' ../Cargo.toml | sed s/\"//g)
TAG="$VERSION"
GH_IMAGE="$GH_REPO:$TAG"
DATE="$(date '+%Y-%m-%d %T%z')"

BUILD_ARGS=(
    "--build-arg" "VCS_REF=${SOURCE_COMMIT:-$(git rev-parse HEAD)}"
    "--build-arg" "VCS_BRANCH=${SOURCE_BRANCH:-$(git symbolic-ref --short HEAD || true)}"
    "--build-arg" "REPO=$GH_REPO"
    "--build-arg" "TAG=$TAG"
    "--build-arg" "DATE=$DATE"
    "--build-arg" "VERSION=$VERSION"
)

if [[ "$CI" = "true" || "$RELEASE" = "true" ]]; then
   PUSH="--push"
fi

echo $PUSH $GH_IMAGE $GH_REPO
if [ "$docker" = "docker" ]; then
    # Can't load the image in the local registry... See https://github.com/docker/roadmap/issues/371
    docker buildx build \
        "${BUILD_ARGS[@]}" \
        $PUSH \
        --pull \
        --platform linux/amd64,linux/arm64 \
        -t $GH_IMAGE \
        -t $GH_REPO:latest \
        -f Dockerfile ../
else
    # Note: to build multi arch images with podman, use these commands:
    # sudo podman run --rm --privileged multiarch/qemu-user-static --reset -p yes
    # podman manifest push --all ghcr.io/trow-registry/trow:0.7.2
    podman manifest create $GH_IMAGE
    podman build \
        "${BUILD_ARGS[@]}" \
        --platform linux/amd64,linux/arm64 \
        --manifest $GH_IMAGE \
        -f Dockerfile ../

fi

echo "→ Image $GH_IMAGE built successfully"
if [[ "$CI" = true ]]
then
    echo "container-image=$GH_IMAGE" >> $GITHUB_OUTPUT
fi
