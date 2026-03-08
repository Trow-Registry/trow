# syntax=docker/dockerfile:1

# Cross compiling from host platform instead of using qemu
FROM --platform=$BUILDPLATFORM docker.io/rust:bullseye AS cross

ARG TARGETARCH
WORKDIR /usr/src/trow

ENV PKG_CONFIG_SYSROOT_DIR=/ \
    SQLX_OFFLINE=true

RUN --mount=type=bind,source=docker/platform.sh,target=/platform.sh \
    /platform.sh

RUN rustup target add $(cat /.platform)

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean &&\
    apt-get update &&\
    apt-get install -y unzip openssl cmake clang $(cat /.compiler)

COPY . .

# Remove project-level cargo config that assumes mold linker is available
RUN rm -f .cargo/config.toml

# Get rid of the "cp" when cargo build --out is stable
RUN --mount=type=cache,target=./target \
    cargo build --release --target $(cat /.platform) &&\
    cp /usr/src/trow/target/$(cat /.platform)/release/trow /usr/src/trow/



FROM debian:bullseye-slim

LABEL org.opencontainers.image.source="https://github.com/trow-registry/trow" \
  org.opencontainers.image.description="Caching Container Registry and Image Management for Kubernetes Clusters" \
  org.opencontainers.image.licenses="Apache-2.0"

RUN groupadd -r -g 2000 trow &&\
    useradd -r -g trow -u 2000 trow

# Note that certs are needed for proxying
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update &&\
    apt-get install -y --no-install-recommends openssl ca-certificates

RUN mkdir -p /data /certs &&\
    chown -R trow /data /certs
# keep this late for concurrency
COPY --from=cross /usr/src/trow/trow /trow
USER trow
ENTRYPOINT ["/trow"]
