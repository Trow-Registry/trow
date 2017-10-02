@0x8e59099839a48161;

struct Message  {
  text @0 :Text;
  number @1 :UInt8;
}


interface MessageInterface {
  list @0 () -> (list: List(Message));
  send @1 (msg: Message) -> ();
}
