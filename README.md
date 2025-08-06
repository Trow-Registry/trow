# Trow

[![Tests](https://github.com/trow-registry/trow/actions/workflows/pr-tests.yaml/badge.svg)](https://github.com/trow-registry/trow/actions/workflows/pr-tests.yaml)

Image management and caching for Kubernetes.

We're building a small registry to make image management in Kubernetes easy.
The Trow Registry runs inside the cluster with very little resources, and is simple to set-up
so it caches every image.

## Use Cases

* Spin up a lightweight registry within Kubernetes
* Cache every image in a cluster when using the proper containerd or cri-o configuration
* Prevent unauthorized images form touching the cluster with the admission webhook

Features include:

- [x] conforms to the [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec) for registries
- [x] controls images running inside the cluster via approve/deny lists
- [x] automagically proxy any registry
- [ ] distributed architecture for HA and scalability _(coming soon)_
- [ ] full auditing and authentication of image access _(planned)_

## Comparison to Other Registries

See [COMPARISON.md](docs/COMPARISON.md).

## Install

A [helm chart is available](./charts/trow).

Note that Trow is currently beta and you can expect to find rough edges.

## Architecture and Design

If you're interested in the design of Trow please take a look at the [Architecture
Guide](docs/ARCHITECTURE.md).

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

## Why "Trow"

"Trow" is a word with multiple, divergent meanings. In Shetland folklore a trow
is a small, mischievous creature, similar to the Scandinavian troll. In England,
it is an old style of cargo boat that transported goods on rivers. Finally, it is
an archaic word meaning "to think, believe, or trust". The reader is free to
choose which interpretation they like most, but it should be pronounced to rhyme
with "brow".
