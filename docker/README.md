Building Trow
=============

The easiest way to build Trow is via Dockerfile. Either run `build.sh` from this directory or run
something similar to following:

```
docker build -f Dockerfile.debug -t trow ../
```

Note that the build context needs to be the root direcotry of the project.

To run tests, use the `Dockerfile.test` image (tests will run as part of the build).
