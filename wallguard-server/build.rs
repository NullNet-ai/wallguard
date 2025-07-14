const WALLGUARD_PROTOBUF_PATH: &str = "./proto/wallguard.proto";
const PROTOBUF_DIR_PATH: &str = "./proto";

fn main() {
    for out_dir in ["./src/protocol", "./libwallguard/src/proto"] {
        tonic_build::configure()
            .out_dir(out_dir)
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
            .compile_protos(&[WALLGUARD_PROTOBUF_PATH], &[PROTOBUF_DIR_PATH])
            .expect("Protobuf files generation failed");
    }
}
