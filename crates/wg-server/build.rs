fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);

    // Server: gRPC server for provisioning/control/data; no CLI socket service.
    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .compile_protos(
            &[
                "../../proto/provisioning.proto",
                "../../proto/control.proto",
                "../../proto/data.proto",
            ],
            &["../../proto/"],
        )?;
    Ok(())
}
