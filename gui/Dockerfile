FROM --platform=${BUILDPLATFORM:-linux/amd64} rust:1.53.0 as builder
WORKDIR /usr/src/gui
RUN rustup target add wasm32-unknown-unknown

COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ \ 
    && echo 'fn main() { println!("caching"); }' > src/lib.rs
RUN cargo build --release

COPY . .
RUN cargo install wasm-pack trunk wasm-bindgen-cli 
RUN trunk build --release

FROM --platform=${BUILDPLATFORM:-linux/amd64} nginx:alpine
COPY --from=builder /usr/src/gui/dist /usr/share/nginx/html
RUN sed -i '/}/i \    application/wasm wasm;' /etc/nginx/mime.types
CMD ["nginx", "-g", "daemon off;"]
