const OUTPUT_DIR: &'static str = "./src/protobuf";
const INCLUDE_PATH: &'static str = "../proto";
const PROTO_FILES: [&'static str; 3] = [
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
        .compile_protos(&PROTO_FILES, &[INCLUDE_PATH])
        .expect("Protobuf files generation failed");
}
