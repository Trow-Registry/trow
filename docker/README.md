Building Trow
=============

The easiest way to build Trow is via Dockerfile. Either run `build.sh` from this directory or run
something similar to following:

```
docker build -f Dockerfile -t trow ..
```

Note that the build context needs to be the root direcotry of the project (*not* the directory with
the Dockerfile).

To run tests, use the `build.sh` script or `Dockerfile.test` image (tests will run as part of the build).

Once issues related to TLS libraries and GRPC have been resolved, a minimal build based on a scratch
image will be added.
