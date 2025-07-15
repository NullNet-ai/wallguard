const OUTPUT_DIR: &str = "./src/protobuf";
const INCLUDE_PATHS: [&str; 2] = ["../proto", "/usr/include"];
const PROTO_FILES: [&str; 3] = [
    "../proto/cli.proto",
    "../proto/commands.proto",
    "../proto/service.proto",
];

fn main() {
    tonic_build::configure()
        .out_dir(OUTPUT_DIR)
        .type_attribute(
            "wallguard_service.PacketsData",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_service.Packet",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_service.SystemResourcesData",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_service.SystemResource",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(&PROTO_FILES, &INCLUDE_PATHS)
        .expect("Protobuf files generation failed");
}
