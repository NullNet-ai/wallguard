fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);

    // Agent: gRPC client for control/provisioning/data; server for CLI socket.
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(
            &[
                "../../proto/provisioning.proto",
                "../../proto/control.proto",
                "../../proto/data.proto",
                "../../proto/cli.proto",
            ],
            &["../../proto/"],
        )?;
    Ok(())
}
