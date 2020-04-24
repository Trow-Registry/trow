# Developing Trow

The easiest way to build and test Trow is by using the [Dockerfiles](./docker/README.md).

Trow is written in [Rust](https://www.rust-lang.org/) and can be compiled, run and tested with the
`cargo` tool: `cargo run` should start a local instance of Trow and `cargo test` should run the test
suite. 

At the moment, we use Rust nightly due to a dependency on the [Rocket](https://rocket.rs/) framework.

Personally, I (Adrian Mouat) use [Visual Studio Code](https://code.visualstudio.com/) with Rust
extensions when developing, but other contributors have sucessfully used Vim and other tools.
