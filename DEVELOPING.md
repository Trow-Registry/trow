# Developing Trow

## Using Docker

The easiest way to build and test Trow is by using the [Dockerfiles](./docker/README.md) and
associated scripts. If you want a local binary, the Trow executable can be copied out of the final
image. The only dependency for building Trow in this way is Docker.

## Using Local Tools

Trow is written in [Rust](https://www.rust-lang.org/). At the moment, we use Rust nightly due to a
dependency on the [Rocket](https://rocket.rs/) framework.

To compile Rust locally, first install [rustup](https://www.rust-lang.org/tools/install) if you
haven't already. Set the compiler default to nightly with `rustup default nightly` and run `rustup
component add rustfmt`. Then run `rustup update` to make sure you're running a version of Rust with
`rustfmt` component. Now you should be able to run `cargo build` from the project root and the Trow
binary will be written to `/target/debug/trow`.

To execute the binary, you can run `cargo run`, which will first recompile Trow if anything has
changed.
