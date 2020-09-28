with import <nixpkgs> {};
let
  pkgs = import <nixpkgs> {};
  date = "2020-09-28";
  mozilla-overlay = fetchFromGitHub {
    owner = "mozilla";
    repo = "nixpkgs-mozilla";
    rev = "9f3df31ebb32ac97e385e07edd954fbf027ca242";
    sha256 = "1swds7y664m04m1grjhypvslxzzwhiby4xqlc0aml62gbyi88jya";
  };
  mozilla = (import mozilla-overlay) pkgs pkgs;
  rust-nightly = mozilla.rustChannelOf { inherit date; channel = "nightly"; };
in
# rustPlatform.buildRustPackage rec {
stdenv.mkDerivation rec {
  name = "trow";
  env = buildEnv { name = name; paths = buildInputs; };
  # src = lib.sourceFilesBySuffices ./. [".lock" ".toml" ".rs"];
  # cargoSha256 = "0q68qyl2h6i0qsz82z840myxlnjay8p1w5z7hfyr8fqp7wgwa9cx";
  buildInputs = [
    pkgconfig
    rust-nightly.rust

    cmake
    perl
    go
    sqlite

    # for .proto generation
    protobuf

    # dev
    valgrind
  ];

  configurePhase = ''true'';

  buildPhase = ''
    echo "Starting the build"
    cargo build -vv
  '';


  shellHook = ''
    >&2 echo Rust Nightly: ${date}
    export PROTOC="${protobuf}/bin/protoc"
  '';
}
