const OUTPUT_DIR: &str = "./src/protobuf";
const INCLUDE_PATHS: [&str; 2] = ["../proto", "/usr/include"];
const PROTO_FILES: [&str; 5] = [
    "../proto/cli.proto",
    "../proto/models.proto",
    "../proto/commands.proto",
    "../proto/service.proto",
    "../proto/tunnel.proto",
];

fn main() {
    tonic_build::configure()
        .out_dir(OUTPUT_DIR)
        .protoc_arg("--experimental_allow_proto3_optional")
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
        .type_attribute(
            "wallguard_models.FilterRule",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_models.NatRule",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_models.NetworkInterface",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_models.Alias",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_models.IpAddress",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_models.SSHConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard_models.Configuration",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(&PROTO_FILES, &INCLUDE_PATHS)
        .expect("Protobuf files generation failed");
}
