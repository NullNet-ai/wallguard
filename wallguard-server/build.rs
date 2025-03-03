const WALLGUARD_PROTOBUF_PATH: &str = "./proto/wallguard.proto";
const PROTOBUF_DIR_PATH: &str = "./proto";

fn main() {
    tonic_build::configure()
        .out_dir("./src/proto")
        .type_attribute(
            "wallguard.Packets",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard.Packet",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard.Log",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "wallguard.Authentication",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(&[WALLGUARD_PROTOBUF_PATH], &[PROTOBUF_DIR_PATH])
        .expect("Protobuf files generation failed");
}
