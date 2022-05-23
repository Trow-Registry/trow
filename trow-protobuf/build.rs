use std::fs;

fn main() {
    fs::create_dir("out").ok();

    tonic_build::configure()
        .out_dir("out")
        .compile(&["src/server.proto"], &["src/"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
