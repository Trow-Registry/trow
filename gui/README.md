# Trow GUI

Trow. The Cloud Native Registry.

-   requirements

    -   trow registry instance running
    -   (optional) export `TROW_REGISTRY_URL` env variable, default assumed at `https://trow.local:8443`
    -   (optional) export `PROXY_PORT` env variable to change the proxy port, default is set to `9001`


-   install dependencies

```
yarn install
```

-   start gui and proxy server

```
yarn start
```

```
http://localhost:9000 - gui
http://localhost:9001 - proxy

```
