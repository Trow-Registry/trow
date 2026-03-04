#!/bin/bash
set -eux

# Used in Docker build to set platform dependent variables

mkdir -p $HOME/.cargo
cat > /root/.cargo/config.toml <<EOF
    [target.aarch64-unknown-linux-gnu]
    linker = "x86_64-linux-gnu-gcc"

    [target.armv7-unknown-linux-gnueabihf]
    linker = "arm-linux-gnueabihf-gcc"

    [target.aarch64-unknown-linux-gnu]
    linker = "aarch64-linux-gnu-gcc"
EOF

case $TARGETARCH in
	"amd64")
		echo "Building for amd64"
		echo "x86_64-unknown-linux-gnu" > /.platform
		echo "gcc-x86-64-linux-gnu" > /.compiler
	;;
	"arm64")
		echo "Building for arm64"
		echo "aarch64-unknown-linux-gnu" > /.platform
		echo "gcc-aarch64-linux-gnu" > /.compiler
	;;
	"arm")
		echo "Building for amd32"
		echo "armv7-unknown-linux-gnueabihf" > /.platform
		echo "gcc-arm-linux-gnuabihf" > /.compiler
	;;
esac
