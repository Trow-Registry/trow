# Developing Trow

## sqlx setup

To develop, build and test Trow, an sqlx development database is required.
The TL;DR is `cargo sqlx database setup`, more information can be found at
<https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md>.

## Using Docker/Podman

The easiest way to build and test Trow is by using the [Containerfiles](./docker/README.md) and
associated scripts. If you want a local binary, the Trow executable can be copied out of the final
image. The only dependency for building Trow in this way is Docker or Podman.

## Using Local Tools

Trow is written in [Rust](https://www.rust-lang.org/).

To compile Rust locally, first install [rustup](https://www.rust-lang.org/tools/install) if you
haven't already. Set the compiler default to nightly with `rustup default nightly` and run `rustup
component add rustfmt`. Then run `rustup update` to make sure you're running a version of Rust with
`rustfmt` component. Now you should be able to run `cargo build` from the project root and the Trow
binary will be written to `/target/debug/trow`.

To execute the binary, you can run `cargo run`, which will first recompile Trow if anything has
changed.

## Running OCI conformance tests locally

```bash
CONT=$(podman create ghcr.io/opencontainers/distribution-spec/conformance:v1.1.0)
podman cp $CONT:/conformance.test .
podman rm $CONT
OCI_ROOT_URL="http://127.0.0.1:8000" OCI_TEST_PULL=1 ./conformance.test
```
