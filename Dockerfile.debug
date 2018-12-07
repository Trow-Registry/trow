#Not a big fan of using nightly, but such is our lot currently
FROM rust:latest as builder

RUN rustup update nightly && rustup default nightly;

RUN apt-get update && apt-get install -y cmake golang unzip
#Add checksum or GPG checking
RUN cd /usr/local && curl -o protoc.zip -sSL https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip && unzip protoc.zip

WORKDIR /usr/src/trow

#First get just the deps
COPY Cargo.toml .
COPY Cargo.lock .

# trow
RUN mkdir src/
RUN echo "fn main() {}" > src/main.rs

# trow-server
COPY lib/server/Cargo.toml lib/server/
RUN mkdir -p lib/server/src
RUN touch lib/server/src/lib.rs

# trow-protobuf
COPY lib/protobuf/Cargo.toml lib/protobuf/
RUN mkdir -p lib/protobuf/src
RUN touch lib/protobuf/src/lib.rs

RUN cargo fetch #This should be cargo build, but it fails as we need build libs first
COPY lib lib
COPY src src
RUN touch src/main.rs
#Do a normal build for debug
RUN cargo build

FROM debian:stable-slim

ARG VCS_REF
LABEL org.opencontainers.image.revision=$VCS_REF \
      org.opencontainers.image.source="https://github.com/ContainerSolutions/trow"

RUN apt-get update \
 && apt-get install -y --no-install-recommends openssl \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/trow/target/debug/trow /trow

COPY install/self-cert /install/self-cert
COPY start-trow.sh /
RUN mkdir --parents /data/layers
RUN mkdir /data/scratch
ENTRYPOINT ["/start-trow.sh"]
