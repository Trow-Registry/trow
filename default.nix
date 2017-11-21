with import <nixpkgs> {};
let
  date = "2017-11-15";
  rust-nightly = mozilla.rustChannels.rustChannelOf { date = date; channel = "nightly"; };
in
stdenv.mkDerivation rec {
  name = "lycaon";
  env = buildEnv { name = name; paths = buildInputs; };
  # src = ./.;
  buildInputs = [
    pkgconfig
    rust-nightly.rust
    capnproto

    # dev
    rustfmt
    valgrind
  ];

  shellHook = ''
    echo Rust Nightly: ${date}
  '';
}
