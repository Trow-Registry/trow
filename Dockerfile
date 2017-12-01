#Not a big fan of using nightly, but such is our lot currently
FROM rustlang/rust:nightly as builder

RUN apt-get update && apt-get install -y musl musl-dev musl-tools
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/lycaon

#Let's grab and install the dependencies first with a bit of a hack...
COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
#Use musl to get a static binary 
RUN cargo build --release --target x86_64-unknown-linux-musl

#Now the real src
COPY build.rs .
COPY src ./src
COPY Rocket.toml .
# Need to touch to alert cargo that things have changed...
RUN touch src/main.rs 
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /usr/src/lycaon/target/x86_64-unknown-linux-musl/release/lycaon /lycaon
COPY Rocket.toml /
#Yeah, it's another hack. Sue me. Note that you need 17.09 to build this
COPY --chown=65534:65534 README.md /data/layers/README.md
COPY --chown=65534:65534 README.md /data/scratch/README.md
USER 65534
EXPOSE 8000
CMD ["/lycaon"]
