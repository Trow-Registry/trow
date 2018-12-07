#Not a big fan of using nightly, but such is our lot currently.
#TODO Use same Dockerfile for release and debug
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

RUN cargo fetch
COPY lib lib
COPY src src
RUN touch src/main.rs
RUN cargo build --release
# So we can use the build target to run tests
COPY tests tests

FROM scratch

COPY --from=builder /usr/src/trow/target/release/trow /trow

#Need to copy c libs
COPY --from=builder /lib/x86_64-linux-gnu/libdl.so.2 /lib/libdl.so.2
COPY --from=builder /lib/x86_64-linux-gnu/librt.so.1 /lib/librt.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libpthread.so.0 /lib/libpthread.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/libgcc_s.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libc.so.6 /lib/libc.so.6
COPY --from=builder /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2
COPY --from=builder /lib/x86_64-linux-gnu/libm.so.6 /lib/libm.so.6

COPY trow-default.toml /
COPY certs /certs
##Yeah, it's another hack. Sue me. Note that you need 17.09+ to build this
COPY --chown=65534:65534 README.md /data/layers/README.md
COPY --chown=65534:65534 README.md /data/scratch/README.md
USER 65534
EXPOSE 8000
ENTRYPOINT ["/trow"]
CMD ["-c", "/trow-default.toml"]
