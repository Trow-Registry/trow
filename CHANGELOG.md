# Changelog

## v0.8.0 (2025-06-18)

* listen on IPv6 socket
* supports image reference with IPv6 host
* perf: use SQL triggers to fill a manifest_blob_assoc table

## v0.7.5 (2025-03-07)

proxy: fix error where the same manifest in a different repo would cause HTTP 404

## v0.7.4 (2025-03-05)

proxy: log error when creating an OCI client fails

## v0.7.3 (2025-02-24)

* Fix broken arm64 container image

## v0.7.2 (2025-02-21)

* Smoke test pulls from ecr instead of docker (#408)
* Remove dead code
* Separate rw and ro sqlite connection pools (faster response times)
* Auto clean stale uploads, orphaned blobs
* Add option `registry_proxies.max_size`:

  This feature deletes older proxied blobs if the disk usage goes above the specified limit (LRU for proxied blobs).

## v0.7.1 (2025-01-29)

Fix: when downloading proxied image, wait for all blobs to be successfully downloaded before saving the manifest in the DB.

## v0.7.0 (2025-01-28)

### BREAKING

Trow now uses a database to store the registry state.
Currently no migration is provided, the `data/` directory needs to be wiped.

### Features

* Trow now uses a database (sqlite) to store the registry state
* Support for the referrers API

### Fixes

* Fix authentication always failing (`invalid username/password`)

### Changes

* `--proxy-registry-config-file` and `--image-validation-config-file` are merged into `--config-file`
* Lots of code refactoring (use sqlite database)

## v0.6.4

Allow mutating webhook to ignore some repos (fix chicken and egg problem)

**Full Changelog**: <https://github.com/Trow-Registry/trow/compare/v0.6.4...v0.6.3>

## v0.6.3

Fix ECR auth

**Full Changelog**: <https://github.com/Trow-Registry/trow/compare/v0.6.3...v0.6.2>

## v0.6.2

* Dedup routes code (using macro)
* Remove gRPC

**Full Changelog**: <https://github.com/Trow-Registry/trow/compare/v0.6.2...v0.6.1>

## v0.6.1

* Move Trow to the Trow-Registry github org !
* Dependencies upgrades

## v0.6.0 - 10/07/2023

Proxy registry: offline mode

## v0.5.2

* fix dockerfile entrypoint
* fix logging, replace log by tracing

## v0.5.0 - 12/06/2023

* Returned cached proxied image even if upstream is offline (#5)
* Refactoring/cleanups
* Fix and improve tests and CI workflows
* Move from rocket to axum

## v0.4.0

* Proxy any registry (not only dockerhub). Based on <https://github.com/Trow-Registry/trow/pull/250> by @sopak

## v0.3.5 - 10/03/2022

### What's Changed

* Update link to kustomize install. by [@ariyonaty](https://github.com/ariyonaty) in <https://github.com/Trow-Registry/trow/pull/284>
* Simplify building of Docker images by [@amouat](https://github.com/amouat) in <https://github.com/Trow-Registry/trow/pull/287>
* Update quick install docs. by [@amouat](https://github.com/amouat) in <https://github.com/Trow-Registry/trow/pull/293>
* Container images are now signed by cosign by [@amouat](https://github.com/amouat) in <https://github.com/Trow-Registry/trow/pull/297>
* Fix ranges by [@amouat](https://github.com/amouat) in <https://github.com/Trow-Registry/trow/pull/304>
* quick-install: Compatibility fixes for Kubernetes v1.22+ by [@MukeshS-hexaware](https://github.com/MukeshS-hexaware) in <https://github.com/Trow-Registry/trow/pull/307>
* Add multiplatform (manifest list) proxying + fix concurrent writes to files + add concurrent download of layers by [@awoimbee](https://github.com/awoimbee) in <https://github.com/Trow-Registry/trow/pull/314>

The recommend way to obtain Trow is to pull the image from the Docker Hub i.e.:

`docker pull containersol/trow:0.3.5`

You can also build from source or use the below binary. The image on the DockerHub has a "fat manifest" and should work for multiple architectures (amd64, armv7 and amd64). The tgz file contains the Helm chart YAML only.

Please be aware that Trow is in an early stage of development and should not be considered fully production ready.

**Full Changelog**: <https://github.com/Trow-Registry/trow/compare/v0.3.4...v0.3.5>

## v0.3.4 - 10/01/2022

### What's Changed

* Rocket 0.5 by [@amouat](https://github.com/amouat) in https://github.com/Trow-Registry/trow/pull/271
* Refactor proxy authentication mechanism by [@sopak](https://github.com/sopak) https://github.com/Trow-Registry/trow/pull/274
* [#168](https://github.com/Trow-Registry/trow/issues/168) doc improvements and editing out of date api versions. by [@KianTigger](https://github.com/KianTigger) in https://github.com/Trow-Registry/trow/pull/276
* Add CLI parameter for setting the log level (`--log-level`) by [@rorymalcolm](https://github.com/rorymalcolm) in https://github.com/Trow-Registry/trow/pull/275
* Support 5 levels of repos (e.g. `docker pull org/a/b/c/d/myimage`)  by [@amouat](https://github.com/amouat) in https://github.com/Trow-Registry/trow/pull/281

The work to use Rocket 0.5 was fairly major and represents a big step forward in terms of speed and scalability.

There were also a bunch of bug fixes, refactorings and doc improvements.

The recommend way to obtain Trow is to pull the image from the Docker Hub i.e.:

```sh
docker pull containersol/trow:0.3.4
```

You can also build from source or use the below binary. The image on the DockerHub has a "fat manifest" and should work for multiple architectures (amd64, armv7 and amd64). The tgz file contains the Helm chart YAML only.

Please be aware that Trow is in an early stage of development and should not be considered fully production ready.

**Full Changelog**: <https://github.com/Trow-Registry/trow/compare/v0.3.3...v0.3.4>

## v0.3.3 - 30/07/2021

Bug-fix release.

* Fix bug where proxy skipped layers
* Refactored code to use traits
* Refactorings to routing code
* Formatting fixes to code
* Updated libraries & dependencies

Thanks to [@sopak](https://github.com/sopak) for the bugfix and [@mathijshoogland](https://github.com/mathijshoogland) for some help and fixes to Helm stuff.

The recommend way to obtain Trow is to pull the image from the Docker Hub i.e.: `docker pull containersol/trow:0.3.3`

You can also build from source or use the below binary. The image on the DockerHub has a "fat manifest" and should work for multiple architectures (amd64, armv7 and amd64). The `tgz` file contains the Helm chart YAML only.

Please be aware that Trow is in an early stage of development and should not be considered fully production ready.

## v0.3.2 - 03/03/2021

Mainly a bug-fix release.

* Added ca-certificates to Docker images to fix [#234](https://github.com/Trow-Registry/trow/issues/234)
* Set content-length header to fix pulling on containerd
* Whole bunch of refactorings and minor fixes
* Updated libraries & dependencies

Thanks to [@mathijshoogland](https://github.com/mathijshoogland) [@alex-glv](https://github.com/alex-glv) [@blaggacao](https://github.com/blaggacao) [@jonathangold](https://github.com/jonathangold) for their contributions!

The recommend way to obtain Trow is to pull the image from the Docker Hub i.e.:

`docker pull containersol/trow:0.3.2`

You can also build from source or use the below binary. The image on the DockerHub has a "fat manifest" and should work for multiple architectures (amd64, armv7 and amd64).

Please be aware that Trow is in an early stage of development and should not be considered fully production ready.

## v0.3.1 - 12/11/2020

Intermediate release with support for proxying the Docker Hub.

This can be used to effectively mitigate issues from Docker Hub unavailability or rate limiting.

Full instructions are here: <https://github.com/Trow-Registry/trow/blob/master/docs/USER_GUIDE.md#proxying-the-docker-hub>

Example usage:

```shell
$ trow --proxy-docker-hub --hub-user amouat --hub-token-file ./.hub_token
Starting Trow 0.3.1-PROXY on 0.0.0.0:8443

**Validation callback configuration

  By default all remote images are denied, and all local images present in the repository are allowed

  These host names will be considered local (refer to this registry): ["0.0.0.0"]
  Images with these prefixes are explicitly allowed: ["k8s.gcr.io/", "docker.io/containersol/trow"]
  Images with these names are explicitly allowed: []
  Local images with these prefixes are explicitly denied: []
  Local images with these names are explicitly denied: []

Docker Hub repositories are being proxy-cached under f/docker/

Trow is up and running!
```

And now pulls such as the following should work:

```shell
$ docker pull localhost:8443/f/docker/debian:latest
latest: Pulling from f/docker/debian
e4c3d3e4f7b0: Pull complete
Digest: sha256:60cb30babcd1740309903c37d3d408407d190cf73015aeddec9086ef3f393a5d
Status: Downloaded newer image for localhost:8443/f/docker/debian:latest
localhost:8443/f/docker/debian:latest
```

## v0.3.0

Trow is a container image registry designed to run inside Kubernetes. Updates since 0.2:

* A new badge that reports on the results of the OCI Conformance Tests (currently all passing)
* Moved to a StatefulSet for deploys to allow for updating and reattaching volumes
* Handle "Foreign Blobs" in manifests (some manifests refer to external resources that shouldn't be stored in the repo, normally for licensing reasons)
* Added a Tag History endpoint
* Support Manifest Lists to allow multi-arch images and ORAS use cases
* Added contributor guidelines
* Added architecture docs
* Changed the UID of the Trow user to 333333 for security reasons
* Various bug fixes

Note that the UID change _may_ be a breaking change. There are some provisions in Kubernetes for dealing with changes to `securityContext` permissions but I am not clear on exactly how they work or if there are differences in versions of Kubernetes. Please be aware of this change and be prepared to update config or permissions on files.

Thanks to [@Pradyumnakashyap](https://github.com/Pradyumnakashyap) for fixing a typo in the Quick Installer, [@Spazzy757](https://github.com/Spazzy757) for hard work on the forthcoming Helm install :fireworks:  and [@iamcaleberic](https://github.com/iamcaleberic) for starting work on readiness/liveness checks.

The recommend way to obtain Trow is to pull the image from the Docker Hub i.e.:

```sh
docker pull containersol/trow:0.3.0
```

You can also build from source or use the below binary. The image on the DockerHub has a "fat manifest" and should work for multiple architectures (amd64, armv7 and amd64).

Please be aware that Trow is in an early stage of development and should not be considered fully production ready.

## v0.2.0 - 18/02/2020

Trow is a container image registry designed to run inside Kubernetes. Updates since 0.1:

* Trow now passes all current [OCI Distribution Specification conformance tests](https://github.com/opencontainers/distribution-spec/tree/master/conformance). 🎉🎉🎉
* The quick installer can now be configured to use a given namespace rather than `kube-public`
* Fundamental changes to the storage system
* Lots of bug fixes and docs updates

This is a breaking update which includes major changes to the way files are stored. Please back-up any images, and wipe the data directory before updating and repushing images.

The recommend way to obtain Trow is to pull the image from the Docker Hub i.e.:

```sh
docker pull containersol/trow:0.2.0
```

Big thanks go to [@mcwienczek](https://github.com/mcwienczek) for helping with several issues. Thanks also to [@2phost](https://github.com/2phost) [@DavidZisky](https://github.com/DavidZisky) [@sjpotter](https://github.com/sjpotter) [@farlop](https://github.com/farlop) for help with various issues and PRs.

You can also build from source or use the below binary. The image on the DockerHub has a "fat manifest" and should work for multiple architectures (amd64, armv7 and amd64).

Please be aware that Trow is in an early stage of development and should not be considered fully production ready.

## v0.1.0 - 21/11/2019

First release of Trow.

Trow is a container image registry designed to run inside Kubernetes. The first version includes:

* push/pull of images
* listing repositories and tags
* basic auth
* ability to act as a validating webhook and accept/deny images from running in the cluster

The recommend way to obtain Trow is to pull the image from the Docker Hub e.g:

```sh
docker pull containersol/trow:0.1.0
```

You can also build from source or use the below binary.

Please be aware that Trow is in an early stage of development and should not be considered fully production ready.
