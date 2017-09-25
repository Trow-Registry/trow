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
COPY src ./src
COPY Rocket.toml .
# Need to touch to alert cargo that things have changed...
RUN touch src/main.rs 
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /usr/src/lycaon/target/x86_64-unknown-linux-musl/release/lycaon /lycaon
CMD ["/lycaon"]