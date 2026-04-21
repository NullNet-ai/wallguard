fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);

    // CLI: gRPC client for the Unix socket agent control service only.
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(
            &["../../proto/cli.proto"],
            &["../../proto/"],
        )?;
    Ok(())
}
