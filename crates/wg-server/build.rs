fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);

    // Ensure wg-ui/dist exists so rust-embed compiles even before trunk has run.
    // In Docker/CI the real trunk output is copied here before cargo build.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dist = manifest.join("../../crates/wg-ui/dist");
    if !dist.exists() {
        std::fs::create_dir_all(&dist)?;
        std::fs::write(
            dist.join("index.html"),
            b"<!DOCTYPE html><html><body>Run trunk build in crates/wg-ui first.</body></html>",
        )?;
    }

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
