# Comparison to Other Registries

Most registries can be categorised into one or more of the following buckets:

 - Public registries such as Docker Hub and Quay.io
 - Cloud Provider registries such as GCR, ECR, ACR etc. 
 - Self-hosted registries such as Docker Distribution and Harbor.

Trow is very much in the self-hosted registry bucket, but has a different focus to the other
solutions. Notably, it is expected that Trow would be used alongside one or more of the other
solutions - from any of the buckets. In this set-up, Trow would provide fast distribution of images
inside clusters, with the other registry providing long-term storage and potentially acting as the
source of images for Trow.

Harbor is one of the most common options at the minute, so it's worth doing a direct comparison.
Harbor builds on-top of Docker Distribution, addresses a very wide range of use cases and supports a
bunch of different storage options. It is made up of a [large set of
services](https://goharbor.io/docs/1.10/install-config/) which provide a lot of functionality. Each
organisation typically installs a single Harbor instance in a central location that is used by
multiple teams and clusters.

By contrast, Trow is designed to be much more lightweight and to run inside each cluster it serves.
It also integrates with the cluster itself, for example by controlling which images can run in the
cluster through a validating webhook - this allows you to say things like "only allow images from
this registry to run or official images from the Docker Hub". A core aim for Trow is to develop
advanced distribution mechanisms, possibly using P2P or similar techniques to transfer images to
nodes as quickly as possibly (meaning containers start up faster).

As mentioned before, we hope and expect to see Trow working alongside other registries, including
Harbor. In this case, the other registry (Harbor/GCR etc) would be a central store of all images and
would keep a full history the images. Trow would run inside each of the clusters in the organisation
and would distribute the working set of images to nodes. Trow would automatically pull images
through from Harbor for use in the cluster. There a lot of potential benefits from this approach:

 - Images can be distributed faster - the Trow registries are local to the cluster which means a
   shorter network trip - they effectively act as a local cache for the central registry.  
 - Less stress on the central registries, which now only needs to serve the Trow registries,
   rather than every node in the cluster. 
 - Individual clusters can have differently configured registries which meet their distinct needs.
   For example, a development cluster may have looser restrictions on where images can come from or
   how to handle vulnerability scans than a production cluster.

Please note that some of these points are based on features currently planned or in development.
