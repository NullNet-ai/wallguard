pub fn main() {
    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .out_dir("./src/daemon")
        .compile_protos(&["../proto/cli.proto"], &["../proto"])
        .expect("Failed to compile CLI proto file.")
}
