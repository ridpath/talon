fn main() {
    // Only compile protos when swarm feature is enabled AND protoc is available
    #[cfg(feature = "swarm")]
    {
        // Check if protoc is available in PATH
        if std::env::var("PROTOC").is_ok() || which::which("protoc").is_ok() {
            println!("cargo:info=Compiling proto files with protoc");
            
            tonic_build::configure()
                .build_server(true)
                .build_client(true)
                .compile(
                    &["proto/swarm.proto"],
                    &["proto/"],
                )
                .expect("Failed to compile proto files");
        } else {
            println!("cargo:warning=protoc not found, using pre-generated proto code");
            println!("cargo:warning=Install protoc to regenerate: https://github.com/protocolbuffers/protobuf/releases");
        }
    }
}
