fn main() {
    tonic_build::configure()
        .out_dir("./src/datastore/generated")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            &["./src/datastore/proto/store.proto"],
            &["./src/datastore/proto"],
        )
        .expect("Protobuf files generation failed");
}
