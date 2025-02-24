# syntax=docker/dockerfile:1

# Cross compiling from host platform instead of using qemu
FROM --platform=$BUILDPLATFORM rust:bookworm AS cross

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
    apt-get install -y unzip openssl $(cat /.compiler)

COPY . .

# Get rid of the "cp" when cargo build --out is stable
RUN --mount=type=cache,target=./target \
    cargo build --release --target $(cat /.platform) &&\
    cp /usr/src/trow/target/$(cat /.platform)/release/trow /usr/src/trow/



FROM --platform=$TARGETPLATFORM debian:bookworm-slim

LABEL org.opencontainers.image.source="https://github.com/trow-registry/trow"
LABEL org.opencontainers.image.description="Caching Container Registry and Image Management for Kubernetes Clusters"
LABEL org.opencontainers.image.licenses="Apache-2.0"

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

ARG VCS_REF
ARG VCS_BRANCH
ARG DATE
ARG VERSION
ARG REPO
ARG TAG
ENV CREATED=$DATE
ENV VCS_REF=$VCS_REF
ENV VCS_BRANCH=$VCS_BRANCH
ENV VERSION=$VERSION

LABEL org.opencontainers.image.created=$DATE \
  org.opencontainers.image.authors="Container Solutions Labs" \
  org.opencontainers.image.url="https://trow.io" \
  org.opencontainers.image.source="https://github.com/trow-registry/trow" \
  org.opencontainers.image.version=$VERSION \
  org.opencontainers.image.revision=$VCS_REF \
  git.branch=$VCS_BRANCH \
  org.opencontainers.image.title="Trow Cluster Registry" \
  repository=$REPO \
  tag=$TAG
