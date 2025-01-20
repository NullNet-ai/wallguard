const NAME_PROTOBUF_PATH: &str = "./proto/name.proto";
const PROTOBUF_DIR_PATH: &str = "./proto";

fn main() {
    tonic_build::configure()
        .out_dir("./src/proto")
        .compile_protos(&[NAME_PROTOBUF_PATH], &[PROTOBUF_DIR_PATH])
        .expect("Protobuf files generation failed");
}
