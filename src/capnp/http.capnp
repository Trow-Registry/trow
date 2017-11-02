@0x8e59099839a48161;

interface Lycaon {

  # TODO This feels like a huge hack
  getMessageInterface @0 () -> (if: MessageInterface);
  getLayerInterface @1 () -> (if: LayerInterface);

  # -- Begin Sample --
  struct Message  {
    text @0 :Text;
    number @1 :UInt8;
  }


  interface MessageInterface {
    list @0 () -> (list: List(Message));
    send @1 (msg :Message) -> ();
    get @2 (num :UInt8) -> (msg :Message);
  }
# -- End Sample --

# -- Layer --
  struct Layer {
    digest @0 :Text;
    name   @1 :Text;
    repo   @2 :Text;
  }

  interface LayerInterface {
    # query an layer existence.
    # This maps directly with the external API.
    layerExists @0 (layer :Layer) -> (result :Bool);

    # Commit a layer to the image.
    layerCommit @1 (layer :Layer) -> (result :Bool);
  }

}
# -- End Layer --
