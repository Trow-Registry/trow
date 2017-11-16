@0x8e59099839a48161;

interface Lycaon {

  # TODO This feels like a huge hack
  getLayerInterface @0 () -> (if: LayerInterface);
  getUuidInterface @1 () -> (if: UuidInterface);

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
  # -- End Layer --


  # -- Begin Uuid --
  interface UuidInterface {
    # Uuid Struct
    struct Uuid {
      uuid @0 :Text;
    }

    # add a uuid to persistent storage
    addUuid @0 (uuid :Uuid) -> (result :Bool);

    # add a uuid to persistent storage
    saveLayer @1 (uuid :Uuid) -> (result :Bool);
  }
  # -- End Uuid --
}
