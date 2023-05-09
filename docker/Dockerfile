# syntax=docker/dockerfile:1.5

#Note we build on host plaftform and cross-compile to target arch
FROM --platform=$BUILDPLATFORM rust:latest as cross

ARG TARGETARCH
WORKDIR /usr/src/trow

ENV PKG_CONFIG_SYSROOT_DIR=/

# should write /.platform and /.compiler
RUN --mount=type=bind,source=docker/platform.sh,target=/platform.sh \
    /platform.sh

RUN rustup component add rustfmt
RUN rustup target add $(cat /.platform)

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt update && \
    apt-get install -y unzip openssl libssl-dev $(cat /.compiler)

COPY Cargo.toml .
COPY Cargo.lock .
COPY .cargo/config .cargo/config
COPY trow-server/ trow-server/
COPY trow-protobuf/ trow-protobuf/
COPY src src

# Get rid of the "cp" when cargo build --out is stable
RUN --mount=type=cache,target=./target \
    cargo build --release --target $(cat /.platform) && \
    cp /usr/src/trow/target/$(cat /.platform)/release/trow /usr/src/trow/

FROM debian:stable-slim
RUN groupadd -r -g 333333 trow && useradd -r -g trow -u 333333 trow

# Note that certs are needed for proxying
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get install -y --no-install-recommends openssl libssl-dev ca-certificates && \
    apt-get clean

COPY utils/self-cert /install/self-cert
COPY start-trow.sh /
RUN mkdir --parents /data/layers && mkdir /data/scratch && mkdir /certs
# keep this late for concurrency
COPY --from=cross /usr/src/trow/trow /trow
RUN chown -R trow /data /certs /install
USER trow
ENTRYPOINT ["/start-trow.sh"]

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
  org.opencontainers.image.source="https://github.com/extrality/trow" \
  org.opencontainers.image.version=$VERSION \
  org.opencontainers.image.revision=$VCS_REF \
  git.branch=$VCS_BRANCH \
  org.opencontainers.image.title="Trow Cluster Registry" \
  repository=$REPO \
  tag=$TAG
