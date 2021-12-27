[![](https://github.com/containersolutions/trow/workflows/Tests/badge.svg)](https://github.com/containersolutions/trow/actions)

[![](https://github.com/containersolutions/trow/workflows/OCI%20Conformance/badge.svg)](https://github.com/containersolutions/trow/actions)

# Trow
Image Management for Kubernetes

We're building an image management solution for Kubernetes (and possibly other orchestrators).
At its heart is the Trow Registry, which runs inside the cluster, is simple to set-up and fully
integrated with Kubernetes, including support for auditing and RBAC.

### Why "Trow"

"Trow" is a word with multiple, divergent meanings. In Shetland folklore a trow
is a small, mischievous creature, similar to the Scandanavian troll. In England,
it is a old style of cargo boat that transported goods on rivers. Finally, it is
an archaic word meaning "to think, believe, or trust". The reader is free to
choose which interpretation they like most, but it should be pronounced to rhyme
with "brow".
 
## Use Cases

The primary goal for Trow is to create a registry that runs within Kubernetes
and provides a secure and fast way to get containers running on the cluster.

A major focus is providing controls for cluster administrators to define which images
can run in the cluster. Trow can prevent unauthorised and potentially insecure or malicious
images from touching your cluster.

Features include:

 - [x] conforms to the [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec) for registries
 - [x] controls images running inside the cluster via approve/deny lists
 - [ ] full auditing and authentication of image access _(in progress)_
 - [ ] distributed architecture for HA and scalability _(planned)_

## Comparison to Other Registries

There is a [short article on how Trow compares to other registries](docs/COMPARISON.md), including Harbor.

## Install

If you want to quickly try out Trow on a development cluster (either local or remote), follow the
[quick install instructions](./QUICK-INSTALL.md).

This screencast shows how quick it is to get started:

[![asciicast](https://asciinema.org/a/48HK88yR4rJw0QuHt2VdkuVZn.svg)](https://asciinema.org/a/48HK88yR4rJw0QuHt2VdkuVZn)

Normal installations and all production installations should follow the
[Kustomize](docs/KUSTOMIZE_INSTALL.md) or [Helm](docs/HELM_INSTALL.md) installation instructions. Note that
this requires a sub-domain that can pointed at the registry.

Note that Trow is currently alpha and you can expect to find rough edges.

## Architecture and Design

If you're interested in the design of Trow please take a look at the [Architecture
Guide](docs/ARCHITECTURE.md). 

## Tests

There is a reasonably large test suite, which can be run with the `docker/test.sh` script.

## User Guide

Work has started on a [User Guide](docs/USER_GUIDE.md). Currently this explains
how to persist images and how to list repositories and tags via curl.

## Contributing

Please take a look at [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to help out and
[DEVELOPING.md](DEVELOPING.md) for how to get started compiling and running Trow. See also the
[Architecture Guide](docs/ARCHITECTURE.md). 

## Code of Conduct

All participants in the Trow project are expected to comply with the [code of
conduct](CODE_OF_CONDUCT.md).

## Notes

- The project currently runs on Rust Nightly.
