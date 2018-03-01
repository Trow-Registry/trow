with import <nixpkgs> {};
let
  pkgs = import <nixpkgs> {};
  date = "2018-02-25";
  mozilla-overlay = fetchFromGitHub {
    owner = "mozilla";
    repo = "nixpkgs-mozilla";
    rev = "661f3f4d8183f493252faaa4e7bf192abcf5d927";
    sha256 = "0g1ig96a5qzppbf75qwll8jvc7af596ribhymzs4pbws5zh7mp6p";
  };
  mozilla = (import mozilla-overlay) pkgs pkgs;
  rust-nightly = mozilla.rustChannelOf { date = date; channel = "nightly"; };
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
  '';
}
