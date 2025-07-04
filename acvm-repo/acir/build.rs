use std::path::Path;

fn main() {
    let (protoc_bin, include_dir) =
        protoc_prebuilt::init("29.3").expect("failed to initialize protoc");

    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("PROTOC", protoc_bin);
    }

    prost_build::compile_protos(
        &[
            // DTOs for a `Program`, which work with the types in `acir.cpp`
            "./src/proto/program.proto",
            // DTOs for the `WitnessStack`, which work with the types in `witness.cpp`
            "./src/proto/acir/witness.proto",
            // A pared down DTO for `Program`, so Barretenberg can ignore the Brillig part.
            // This is only included to make sure it compiles.
            "./src/proto/acir/program.proto",
        ],
        &[Path::new("./src/proto"), include_dir.as_path()],
    )
    .expect("failed to compile .proto schemas");
}
