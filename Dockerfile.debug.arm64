#Not a big fan of using nightly, but such is our lot currently
#FROM arm64v8/debian:stable-slim
FROM multiarch/debian-debootstrap:arm64-stretch-slim as builder

RUN apt update && apt install -y curl
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN set -eux; \
    url="https://static.rust-lang.org/rustup/dist/aarch64-unknown-linux-gnu/rustup-init"; \
    wget "$url"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain nightly; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;
RUN apt-get update && apt-get install -y cmake golang g++
#RUN rustup target add aarch64-unknown-linux-gnu || true

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
COPY src src
COPY lib lib
RUN touch src/main.rs
RUN cargo build

FROM arm64v8/debian:stable-slim
#
COPY --from=builder /usr/src/trow/target/debug/trow /trow
#
##Need to copy c libs
##COPY --from=builder /lib/x86_64-linux-gnu/libdl.so.2 /lib/libdl.so.2
##COPY --from=builder /lib/x86_64-linux-gnu/librt.so.1 /lib/librt.so.1
##COPY --from=builder /lib/x86_64-linux-gnu/libpthread.so.0 /lib/libpthread.so.0
##COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/libgcc_s.so.1
##COPY --from=builder /lib/x86_64-linux-gnu/libc.so.6 /lib/libc.so.6
##COPY --from=builder /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2
##COPY --from=builder /lib/x86_64-linux-gnu/libm.so.6 /lib/libm.so.6
#
COPY Trow.toml /
COPY trow-default.toml /
#Yeah, it's another hack. Sue me. Note that you need 17.09 to build this
COPY --chown=65534:65534 README.md /data/layers/README.md
COPY --chown=65534:65534 README.md /data/scratch/README.md
USER 65534
EXPOSE 8000
ENTRYPOINT ["/trow"]
CMD ["-c", "/trow-default.toml"]
