#Not a big fan of using nightly, but such is our lot currently
FROM rust:latest as builder

RUN rustup update nightly && rustup default nightly;
RUN apt-get update && apt-get install -y unzip

WORKDIR /usr/src/trow

#First get just the deps for caching.
COPY Cargo.toml .
COPY Cargo.lock .

RUN mkdir src/
RUN echo "fn main() {}" > src/main.rs

# trow-server
COPY lib/server/Cargo.toml lib/server/
RUN mkdir -p lib/server/src
RUN touch lib/server/src/lib.rs

# trow-protobuf
COPY lib/protobuf/Cargo.toml lib/protobuf/
RUN echo "fn main() {}" > lib/protobuf/build.rs
RUN mkdir -p lib/protobuf/src
RUN touch lib/protobuf/src/lib.rs

RUN cargo build # should cache all build deps

COPY lib lib
COPY src src
RUN cargo build -Z unstable-options --out-dir ./out

FROM debian:stable-slim

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
