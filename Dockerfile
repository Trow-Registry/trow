#Not a big fan of using nightly, but such is our lot currently
FROM rustlang/rust:nightly

WORKDIR /usr/src/lycaon

#Let's grab and install the dependencies first with a bit of a hack...
COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs 
RUN cargo build 

#Now the real src
COPY src ./src
COPY Rocket.toml .
# Need to touch to alert cargo that things have changed...
RUN touch src/main.rs 
RUN cargo build

CMD ["/usr/local/cargo/bin/cargo", "run"]
