#!/usr/bin/env bash
set -exo pipefail

# change to directory with script so we know where project root is
src_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$src_dir"

# Use GITHUB_REPOSITORY if available (e.g. in CI), otherwise default to upstream
GH_REPO="$(echo "ghcr.io/${GITHUB_REPOSITORY:-trow-registry/trow}" | tr '[:upper:]' '[:lower:]')"


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

VCS_REF="${SOURCE_COMMIT:-$(git rev-parse HEAD)}"
VCS_BRANCH="${SOURCE_BRANCH:-$(git symbolic-ref --short HEAD || true)}"

sudo podman run --privileged --rm docker.io/tonistiigi/binfmt --install all

podman manifest create -a $GH_IMAGE
for platform in linux/arm64 linux/amd64; do
    podman build \
        --label "org.opencontainers.image.created=$DATE" \
        --label "org.opencontainers.image.authors=Container Solutions Labs" \
        --label "org.opencontainers.image.url=https://trow.io" \
        --label "org.opencontainers.image.source=https://github.com/trow-registry/trow" \
        --label "org.opencontainers.image.version=$VERSION" \
        --label "org.opencontainers.image.revision=$VCS_REF" \
        --label "git.branch=$VCS_BRANCH" \
        --label "org.opencontainers.image.title=Trow Cluster Registry" \
        --label "repository=$GH_REPO" \
        --label "tag=$TAG" \
        --platform $platform \
        --manifest $GH_IMAGE \
        -f Dockerfile ../
done

# podman manifest push ghcr.io/trow-registry/trow:0.7.2

echo "→ Image $GH_IMAGE built successfully"
if [[ "$CI" = true ]]
then
    echo "container-image=$GH_IMAGE" >> $GITHUB_OUTPUT
fi
