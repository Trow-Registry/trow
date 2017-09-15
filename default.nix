with import <nixpkgs> {};
stdenv.mkDerivation rec {
  name = "lycaon";
  env = buildEnv { name = name; paths = buildInputs; };
  buildInputs = [
    latest.rustChannels.nightly.rust
  ];
}
