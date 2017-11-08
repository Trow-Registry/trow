@0x8e59099839a48161;

interface Lycaon {

  # TODO This feels like a huge hack
  getLayerInterface @0 () -> (if: LayerInterface);

# -- Layer --
  struct Layer {
    digest @0 :Text;
    name   @1 :Text;
    repo   @2 :Text;
  }

  struct LayerResult {
    exists @0 :Bool;
    length   @1 :UInt64;
  }

  interface LayerInterface {
    # query an layer existence.
    # This maps directly with the external API.
    layerExists @0 (layer :Layer) -> (result :LayerResult);

    # Commit a layer to the image.
    layerCommit @1 (layer :Layer) -> (result :Bool);
  }

}
# -- End Layer --
