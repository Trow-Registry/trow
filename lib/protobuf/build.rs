fn main() {
    //tonic_build::compile_protos("src/server.proto").unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    
    tonic_build::configure()
        .out_dir("out")
        .compile(
            &["src/server.proto"],
            &["src/"]
        ).unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
        
}
