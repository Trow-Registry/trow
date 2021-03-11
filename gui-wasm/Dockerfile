FROM --platform=${BUILDPLATFORM:-linux/amd64} rust:1.50.0 as builder
RUN curl -fsSL https://deb.nodesource.com/setup_15.x | bash - && \
    apt update && \
    apt install -y nodejs && \
    npm install --global yarn

WORKDIR /usr/src/gui
COPY . .
RUN cargo install wasm-pack
RUN yarn install & yarn build

FROM --platform=${BUILDPLATFORM:-linux/amd64} nginx 
COPY --from=builder /usr/src/gui/dist /usr/share/nginx/html
RUN sed -i '/}/i \    application/wasm wasm;' /etc/nginx/mime.types
CMD ["nginx", "-g", "daemon off;"]
