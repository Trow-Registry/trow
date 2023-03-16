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

### Configuring TLS and Routing (required to run test suite on host)

Running the tests is a little more complicated as you need to set the domain `trow.test` to point to
your local machine and set-up TLS certificates. Instead of doing this you might find it easier to
use the slower Docker method instead, by simply running the `docker/test.sh` script.

If you still want to run the tests locally, you will first need to configure routing. On my Linux
laptop I do this by adding the line `127.0.0.1 trow.test` to `/etc/hosts`. I believe this will also
work on MacOS and there is a similar solution for Windows (which uses a different directory). Rather
than do this manually you may want to try the [hostctl tool](https://github.com/guumaster/hostctl).

After this, we need to create a TLS certificate for testing. You can accomplish this using the
[mkcert tool](https://github.com/FiloSottile/mkcert).

You should be now be able to run `cargo test` to run the test suite.

## Editor

Personally, I (Adrian Mouat) use [Visual Studio Code](https://code.visualstudio.com/) with Rust
extensions when developing, but other contributors have sucessfully used Vim and other tools.

## Developing with Nix

Trow can also be developed with [Nix](https://nixos.org/guides/install-nix.html). Nix will create
and manage a development environment that includes the correct Rust Nightly version and all other
tools used in development. This environment can be entered manually by running `nix-shell` within
the root of the Trow project or automatically with [direnv](https://direnv.net/).

### Editor integration with Nix

Any editor with `direnv` integration will have access to the Nix development environment.

Users of Visual Studio Code should set the Rust extension option
`rust-client.disableRustup` to `true` within this workspace to ensure that vscode uses the Rust
tools provided by Nix.
