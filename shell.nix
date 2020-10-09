{ date ? "2020-09-28" }:
let
  # To update nixpkgs or mozilla overlay pin:
  #   1. Update `url` to the desired rev
  #   2. Determine required sha256 by running `nix-prefetch-url --unpack <URL>`
  nixpkgs = fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/0cfe5377e8993052f9b0dd56d058f8008af45bd9.tar.gz";
    sha256 = "0i3ybddi2mrlaz3di3svdpgy93zwmdglpywih4s9rd3wj865gzn1";
  };
  moz-overlay = import (fetchTarball {
    url = "https://github.com/mozilla/nixpkgs-mozilla/archive/9f3df31ebb32ac97e385e07edd954fbf027ca242.tar.gz";
    sha256 = "1swds7y664m04m1grjhypvslxzzwhiby4xqlc0aml62gbyi88jya";
  });
  pkgs = import
    (fetchTarball {
      # TODO: change to upstream once merged: https://github.com/numtide/devshell/pull/33
      url = "https://github.com/numtide/devshell/archive/f5fe9d89ec96814445c17a21baf29de404c8fd65.tar.gz";
      sha256 = "1nn2q4x4pjm1c72apa5wn0m7iibi2r7l086nc2q18myc4q72n63w";
    }) {
    inherit nixpkgs; overlays = [
    moz-overlay
    (import ./dev/shell/overlay.nix)
  ];
  };
in
with pkgs;
mkDevShell {
  imports = [
    (mkDevShell.importTOML ./devshell.toml)
    ./dev/shell/options.nix
  ];
  packages = [
    (rustChannelOf { inherit date; channel = "nightly"; }).rust
  ];
  bash.extra = ''
    export PROTOC=$(which protoc)
    >&2 echo Rust Nightly: ${date}
  '';
}
