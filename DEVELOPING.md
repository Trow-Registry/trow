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

### Running the Test Suite

Running the tests is a little more complicated as you need to set the domain `trow.test` to point to
your local machine and set-up TLS certificates. Instead of doing this you might find it easier to
use the slower Docker mehod instead, by simply running the `docker/test.sh` script. 

If you still want to run the tests locally, you will first need to configure routing. On my Linux
laptop I do this by adding the line `127.0.0.1 trow.test` to `/etc/hosts`. I believe this will also
work on MacOS and there is a similar solution for Windows (which uses a different directory). 

After this, we need to create a TLS certificate for testing. The easiest way to generate this is to
use the `quick-install/self-cert/make-certs.sh` script (change to the `self-cert` directory before
executing the script). This will create `domain.key` and `domain.crt`. Create a `certs` folder in
the project top level and copy both files there. 

You should be now be able to run `cargo test` to run the test suite.

## Editor

Personally, I (Adrian Mouat) use [Visual Studio Code](https://code.visualstudio.com/) with Rust
extensions when developing, but other contributors have sucessfully used Vim and other tools.
