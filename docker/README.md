# Building Trow

The easiest way to build Trow is via Dockerfile. From this directory, either run `build.sh` or run
something similar to following:

```
docker build -f Dockerfile -t trow ..
```

Note that the build context needs to be the root directory of the project (*not* the directory with
the Dockerfile).

To run tests, use the `build.sh` script or `Dockerfile.test` image (tests will run as part of the build).

Once issues related to TLS libraries have been resolved, a minimal build based on a scratch image
will be added.

## Mulitplatform Builds

There are several ways to produce multiplatform builds with Docker:

 1. Build directly on the target hardware.
 2. Use Docker multiplatform support e.g. `--platform` argument available with buildx to produce
    images for other platforms. This uses QEMU internally to emulate the target platform. In
    practice, I hit issues with this solution, seemingly because of bugs in QEMU and interactions
    with multi-stage builds.
 3. Use Rust cross-compilation to produce a binary for the target platform and copy into a base
    image for the target platform. This requires a bit more configuration, but does work. When
    targetting a low-powered platform (e.g. Raspberry Pi), this option may be considerably faster
    than building directly on the hardware or using emulation.

There are two Dockerfiles for approach number 3 `Dockerfile.armv7` and `Dockerfile.arm64`, which you
can use directly, or as a guide building for other platforms. To use these Dockerfiles, run:

```
docker buildx build --pull --load -t trow:armv7 -f Dockerfile.armv7 --platform linux/arm/v7 ../
```

This assumes you have a recent version of Docker with buildkit installed.

Note that `--pull` avoids an issue whereby Docker can use the wrong base image and `--load` puts the
image into the host Docker image cache.

If you get an error about an unsupported platform, you may need to install binfmt handlers. This can
be done for common platforms with `docker run --privileged --rm
docker/binfmt:a7996909642ee92942dcd6cff44b9b95f08dad64` (also see [qus](https://github.com/dbhi/qus)
for an alternative approach and explanation of what is happening here). Restart docker or create a
new builder instance after doing this.

