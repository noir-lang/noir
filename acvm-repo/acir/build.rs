use std::path::Path;

fn main() {
    let (protoc_bin, include_dir) =
        protoc_prebuilt::init("29.3").expect("failed to initialize protoc");

    std::env::set_var("PROTOC", protoc_bin);

    prost_build::compile_protos(
        &[
            "./src/proto/program.proto",
            // This is only included to make sure it compiles.
            // A separate compilation is needed to turn it into C++ code.
            "./src/proto/acir/program.proto",
        ],
        &[Path::new("./src/proto"), include_dir.as_path()],
    )
    .expect("failed to compile .proto schemas");
}
