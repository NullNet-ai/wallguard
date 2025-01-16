const TRAFFIC_MONITOR_PROTOBUF_PATH: &str = "./proto/traffic_monitor.proto";
const PROTOBUF_DIR_PATH: &str = "./proto";

fn main() {
    tonic_build::configure()
        .out_dir("./src/proto")
        .compile_protos(&[TRAFFIC_MONITOR_PROTOBUF_PATH], &[PROTOBUF_DIR_PATH])
        .expect("Protobuf files generation failed");
}
