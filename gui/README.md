# Trow GUI -wasm

Trow. The Cloud Native Registry.

-   pre-requisites
    > Node v15+ 

    > Yarn 1.22.10+ 

    > Rust 1.51.0
    
    > [Wasm-pack](https://rustwasm.github.io/wasm-pack/installer/#)

-   requirements

    -   Trow registry instance running **with Cross-Origin Resource Sharing(CORS) support enabled**.
        
        `--enable-cors --allow-cors-methods '*' --allow-cors-headers '*' --allow-cors-credentials --allow-cors-origin '*'`
   
    - Current GUI CORS requirements:
        
        - Methods: `GET`, `OPTIONS`     

        - Headers `Content-Type`     
        
    -   [Install](https://rustwasm.github.io/wasm-pack/installer/#) `wasm-pack` for your platform 
    -   Add cargo bin path - `export PATH="$PATH:$HOME/.cargo/bin"` - to `~/.bashrc` or equivalent

<!-- 

cargo +nightly install miniserve
wasm-pack build --target web --out-name wasm --out-dir ./static
miniserve ./static --index index.html

 -->

-   start gui and update registry endpoint on the settings page, default set to: `https://0.0.0.0:8443`

```
    > yarn serve
```


``` 
 http://localhost:9000 - gui

```
