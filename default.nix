with import <nixpkgs> {};
stdenv.mkDerivation rec {
  name = "lycaon";
  env = buildEnv { name = name; paths = buildInputs; };
  buildInputs = [
    pkgconfig
    latest.rustChannels.nightly.rust
    capnproto

    # dev
    rustfmt
  ];
}
