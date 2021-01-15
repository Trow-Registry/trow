# Trow GUI

Trow. The Cloud Native Registry.

-   requirements

    -   trow registry instance running
    -   export `TROW_REGISTRY_URL` env variable, default assumed at https://trow.local:8443
    -   add code below to `trow/src/lib.rs` below API header fairing to allow CORS

    ```
     .attach(fairing::AdHoc::on_response(
                  "CORS dev",
                  |_, resp| {
                      resp.set_raw_header("Access-Control-Allow-Origin", "*");
                  },
              ))
    ```

-   install dependencies

```
yarn install
```

-   start backend

```
yarn start
```

-   available at

```
http://localhost:9000
```
