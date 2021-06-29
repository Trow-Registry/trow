# Trow GUI 

Trow GUI component.

-   pre-requisites


    > Rust 1.53.0
    -   Add cargo bin directory to PATH to `~/.bashrc` or equivalent
        
        `export PATH="$PATH:$HOME/.cargo/bin"` 

    -   Trow registry instance running **with Cross-Origin Resource Sharing(CORS) support enabled**.
        
        `cargo run -- --enable-cors `

    - Install wasm-pack,trunk and wasm-bindgen-cli

        `cargo install wasm-pack trunk wasm-bindgen-cli`
   
    
-   start gui and update registry endpoint on the settings page, default set to:  `https://0.0.0.0:8443`

    `trunk serve`

-  gui will be available at:
    
    >http://localhost:8080 

