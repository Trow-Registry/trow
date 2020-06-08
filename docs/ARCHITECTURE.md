# Trow Architecture

The following diagram depicts Trow running standalone (outside of a cluster).

![](diagrams/trow_arch.png)

Note:

 1. All interaction with clients (normally the Docker Daemon, but also podman, CI/CD tools etc) is
    currently via a RESTful (ish) API interface. The interface is defined by the OCI Distribution
    spec, although Trow adds a few features of its own.

 2. The architecture is split into a "Front End" and a "Back End" The Front End deals with HTTP/S
    requests and makes gRPC calls to the "Back End" for handling registry operations like saving or
    retrieving blobs. 

    At the moment, this split doesn't buy us much, but hopefully in the future it
    will allow more flexible deployment models for Trow (for example a single Front End serving
    multiple Back Ends, or a Front End talking to a Back End on a remote node, or new interface
    types for clients like [gRPC](https://www.grpc.io/)).
    
    At the moment, both the Front End and Back End are compiled into one executable. 

    Yes, Front End and Back End are bad terms, please feel free to suggest alternatives.

 3. Trow saves image data to file. Currently, we don't have any options to use different storage
    like S3.  This still allows a considerable deal of flexibility as it can be backed by multiple
    volume types, but at the same time we avoid the complexity of handling the error states of
    remote storage such as S3. 

    At the moment, the Back End hands file pointers to the Front End to read data from and transfer
    to clients. In the future this could be replaced with other methods e.g. a network stream to
    read from. This is the reason for the dashed line between the file system and the Front End

Trow is implemented in [Rust](https://www.rust-lang.org/). The Front End currently uses the
[Rocket](https://rocket.rs/) web framework, but this may change in the future. The gRPC
communication is handled via Tonic. 

## Typical Kubernetes Deployment

The standard install will result in a deployment like this:

![](diagrams/standard_kubernetes_install.png)

Trow data is backed to a volume, for example Google Persistent Disk. A
[StatefulSet](https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/) rather than a
Deployment is used. This is required to handle updating Trow and reattaching the volume correctly.
All clients, including Kubernetes Nodes themselves connect to the registry via the Ingress. This
requires the Ingress to be provisioned with a certificate in some way e.g. cert-manager or Google
Managed Certificate.

The quick install takes a slightly different approach, with Kubernetes CA signed TLS certificates
and a NodePort for ingress.  Routing in this case is achieved by editing `/etc/hosts` on the nodes
and client as well as adding the Trow cert to the appropriate stores. This works well for testing,
but is a hack that shouldn't be used in production.

## Advanced Distribution Deployment

The plan for the future is to have something more like this:

![](diagrams/advanced_distribution.png)

Every (or most) nodes run an instance of the Trow Back End. These instances communicate and share
files with each other in a P2P style (similar to BitTorrent). This should provide an enormous speed
in up in image deployment time for the cluster. It should also be designed to place minimal extra
load on nodes.

There is a single (or perhaps several to allow for HA) Front End instances for talking to clients. 

We are considering using a [CRDT](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type)
for handling distributed state between Trow instances.

## Upload/Download Tracking and File Layout

![](diagrams/trow_fs.png)

When a client uploads an image, it will begin by uploading all the layers that are not present in
the registry. These uploads are given a UUID and tracked in the `scratch` directory. When an upload
completes, the digest is checked and it is copied to the `blobs` directory before being removed from
`scratch`. The digest is used as the file name. All digests at the moment are SHA256 hashes of the
content, but note that other algorithms could be used in the future - hence the existence of the
`sha256` parent directory.  Once the layers are uploaded, the manifest is uploaded in the same
manner. The manifest is then checked by Trow to make sure all the layers are available, before a
file named after the tag is copied to a directory under manifests e.g. `manifests/redis/latest` or
`manifests/containersol/trow/default`. The contents of the file are the digest of the manifest and
the current date. The manifest itself can then be found by looking up the digest in the `blobs`
directory. If the tag file already exists, the new digest and date are written as the first line in
the file. Previous entries are kept for auditing and the "tag history" endpoint.

So this means:

 - files in the `blobs` directory represent not just image layers, but also manifests and config
   data referred to from manifests.
 - the files in scratch _are not_ digests. They are UUIDs used for temporary tracking of uploads.
 - the manifests folder more or less indexes the blobs; it lets us find the data associated with a
   named tag.
 - doing a "GC sweep" to get rid of unused blobs means going through the manifests directory and
   creating an inventory of digests referred to by the manifests. Any blobs that aren't on the final
   list can be safely deleted without breaking an image. The current plan is to avoid doing this in
   a "pause-the-world" GC pass and instead continually ensure the file system is synchronised.

