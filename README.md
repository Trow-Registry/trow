# Trow
Distributed Cluster Registry

[![Build Status](https://travis-ci.org/ContainerSolutions/trow.svg?branch=master)](https://travis-ci.org/ContainerSolutions/trow)

We're building a cluster-first registry for Kubernetes (and possibly other orchestrators).

### Why "Trow"

"Trow" is a word with multiple, divergent meanings. In Shetland folklore a trow
is a small, mischevious creature, similar to the Scandanavian troll. In England,
it is a old sytle of cargo boat that transported goods on rivers. Finally, it is
an archaic word meaning "to think, believe, or trust". The reader is free to
choose which interpretation they like most, but it should be pronounced to rhyme
with "brow".
 
## Install

See [INSTALL.md](./INSTALL.md). Note that it's currently very alpha software.

## Use Cases

The primary goal for Trow is to create a registry that runs within Kubernetes
and provides a secure and fast way to get containers running on the cluster.

We hope to make it possible for Kubernetes operators to verify and control the
images that are run on their clusters. Proposed features include:

 - allowing operations such as approve/deny lists for images and external registries
 - auditing and authentication of image access 
 - distributed architecture for HA and scalability
 
## Notes

- The project currently runs on Rust Nightly.
