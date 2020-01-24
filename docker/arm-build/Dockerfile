#Not a big fan of using nightly, but such is our lot currently
FROM rust:latest as builder

RUN rustup update nightly && rustup default nightly && \
    rustup component add rustfmt && rustup target install armv7-unknown-linux-gnueabihf && \
    rustup toolchain install nightly-armv7-unknown-linux-gnueabihf
RUN apt-get update && apt-get install -y unzip gcc-arm-linux-gnueabihf
WORKDIR /usr/src/trow

COPY Cargo.toml .
COPY Cargo.lock .
COPY .cargo/config .cargo/config

COPY lib lib
COPY src src

# Must build protobuf first for generated code
RUN cd lib/protobuf && cargo build --target armv7-unknown-linux-gnueabihf
RUN cargo build --target armv7-unknown-linux-gnueabihf -Z unstable-options --out-dir ./out

FROM --platform=linux/arm/v7 debian:stable-slim

RUN groupadd -r -g 999 trow && useradd -r -g trow -u 999 trow

RUN apt-get update \
  && apt-get install -y --no-install-recommends openssl libssl-dev \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/trow/out/trow /trow

COPY quick-install/self-cert /install/self-cert
COPY start-trow.sh /
RUN mkdir --parents /data/layers && mkdir /data/scratch && mkdir /certs
RUN chown -R trow /data /certs /install
USER trow
ENTRYPOINT ["/start-trow.sh"]

ARG VCS_REF
ARG DATE
ARG VERSION
ARG REPO
ARG TAG
ENV CREATED=$DATE

LABEL org.opencontainers.image.created=$DATE \
  org.opencontainers.image.authors="Container Solutions Labs" \
  org.opencontainers.image.url="https://trow.io" \
  org.opencontainers.image.source="https://github.com/ContainerSolutions/trow" \
  org.opencontainers.image.version=$VERSION \
  org.opencontainers.image.revision=$VCS_REF \
  org.opencontainers.image.title="Trow Cluster Registry" \
  repository=$REPO \
  tag=$TAG
