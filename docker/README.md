#Building Trow

The easiest way to build Trow is via Dockerfile. Either run `build.sh` from this directory or run
something similar to following:

```
docker build -f Dockerfile -t trow ..
```

Note that the build context needs to be the root direcotry of the project (*not* the directory with
the Dockerfile).

To run tests, use the `build.sh` script or `Dockerfile.test` image (tests will run as part of the build).

Once issues related to TLS libraries have been resolved, a minimal build based on a scratch image
will be added.

## Mulitplatform Builds

There are several ways to produce multiplatform builds with Docker:

 1. Build directly on the target hw
 2. Use Docker multiplatform support e.g. `--platform` argument available with buildx to produce
   images for other platforms
 3. Use Rust cross-compilation to produce a binary for the target platform and copy into the target
    image

When targetting a low-powered platform (e.g. Raspberry Pi), option 3 will generally be fastest. The
`Dockerfile.armv7` file takes this approach and can be used as a guide for building for other
platforms. 
