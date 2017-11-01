@0x8e59099839a48161;

interface OutgoingHttp {
    newSession @0 (baseUrl :Text) -> (session :HttpSession);
}

interface HttpSession {
    get @0 (path :Text) -> (responseCode :UInt32, body :Data);
    post @1 (path :Text, body :Data) -> (responseCode: UInt32);
}


const pizza :Text = "Hamish was here...";

struct Message  {
  text @0 :Text;
  number @1 :UInt8;
}


interface MessageInterface {
  list @0 () -> (list: List(Message));
  send @1 (msg :Message) -> ();
  get @2 (num :UInt8) -> (msg :Message);
}

const test :Text = "hello";

const mymessage :Text = "Hello there";
