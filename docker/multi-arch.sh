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

sudo podman run --rm --privileged multiarch/qemu-user-static --reset -p yes
podman manifest create -a $GH_IMAGE

for target in linux/arm64:aarch64-unknown-linux-gnu linux/amd64:x86_64-unknown-linux-gnu; do
    platform=$(echo $target | cut -d ':' -f 1)
    rust_target=$(echo $target | cut -d ':' -f 2)

    echo "-> $rust_target"
    cross build --release --target $rust_target
    podman build \
        "${BUILD_ARGS[@]}" \
        --platform $platform \
        --manifest $GH_IMAGE \
        -f Dockerfile ../target/$rust_target/release/
done

if [[ "$CI" = "true" || "$RELEASE" = "true" ]]; then
    podman manifest push --all $GH_IMAGE
    echo "container-image=$GH_IMAGE" >> $GITHUB_OUTPUT
fi
