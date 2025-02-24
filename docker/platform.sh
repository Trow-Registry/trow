#!/bin/bash

# Used in Docker build to set platform dependent variables

case $TARGETARCH in
	"amd64")
		echo "Building for amd64"
		echo "x86_64-unknown-linux-gnu" > /.platform
		echo "clang" > /.compiler
		mold_arch="x86_64"
		curl -L https://github.com/rui314/mold/releases/download/v2.2.0/mold-2.2.0-x86_64-linux.tar.gz -o mold.tar.gz
		tar -xzf mold.tar.gz
		cp -rl mold*/* /usr
		rm -rf mold*
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
