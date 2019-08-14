# Trow
Image Management for Kubernetes

[![Build Status](https://travis-ci.org/ContainerSolutions/trow.svg?branch=master)](https://travis-ci.org/ContainerSolutions/trow)

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
 
## Install

See [INSTALL.md](./INSTALL.md). Note that Trow is currently alpha and you can expect to find rough edges.
This screencast shows how quick it is to get started:

[![asciicast](https://asciinema.org/a/48HK88yR4rJw0QuHt2VdkuVZn.svg)](https://asciinema.org/a/48HK88yR4rJw0QuHt2VdkuVZn)

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
