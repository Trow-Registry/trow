# Trow GUI -wasm

Trow. The Cloud Native Registry.

-   requirements

    -   Trow registry instance running
    -   (optional) export `TROW_REGISTRY_URL` env variable, default assumed at `https://trow.local:8443`
    -   (optional) export `PROXY_PORT` env variable to change the proxy port, default is set to `9001`

<!-- 

cargo +nightly install miniserve
wasm-pack build --target web --out-name wasm --out-dir ./static
miniserve ./static --index index.html

 -->

-   start gui and proxy server

```
yarn serve
```

```
http://localhost:9000 - gui
http://localhost:9001 - proxy

```
