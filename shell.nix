{ date ? "2020-09-28" }:
let
  # To update nixpkgs or mozilla overlay pin:
  #   1. Update `url` to the desired rev
  #   2. Determine required sha256 by running `nix-prefetch-url --unpack <URL>`
  pkgs = import (fetchTarball {
   url = "https://github.com/NixOS/nixpkgs/archive/0cfe5377e8993052f9b0dd56d058f8008af45bd9.tar.gz";
   sha256 = "0i3ybddi2mrlaz3di3svdpgy93zwmdglpywih4s9rd3wj865gzn1";
  });
  moz-overlay = import (fetchTarball {
    url = "https://github.com/mozilla/nixpkgs-mozilla/archive/9f3df31ebb32ac97e385e07edd954fbf027ca242.tar.gz";
    sha256 = "1swds7y664m04m1grjhypvslxzzwhiby4xqlc0aml62gbyi88jya";
  });
in
with pkgs { overlays = [ moz-overlay ]; };
mkShell {
  buildInputs = [
    (rustChannelOf { inherit date; channel = "nightly"; }).rust

    pkgconfig
    cmake
    perl
    go
    sqlite
    protobuf
    openssl
    valgrind
  ];

  shellHook = ''
    >&2 echo Rust Nightly: ${date}
    export PROTOC="${protobuf}/bin/protoc"
  '';
}
