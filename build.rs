extern crate capnpc;

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("src/capnp")
        .file("src/capnp/http.capnp")
        .run().expect("schema compiler command");
}
