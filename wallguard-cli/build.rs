pub fn main() {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .out_dir("./src")
        .compile_protos(&["../proto/cli.proto"], &["../proto"])
        .expect("Failed to compile CLI proto file.")
}
