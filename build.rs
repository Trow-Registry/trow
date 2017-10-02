extern crate capnpc;

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("capnp")
        .file("capnp/index.capnp")
        .run().expect("schema compiler command");
}
