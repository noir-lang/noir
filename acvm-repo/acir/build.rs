use std::path::Path;

fn main() {
    // This version downloads a configurable protoc version during the build, a separate one each time our code changes.
    // Despite having the GITHUB_TOKEN, it often gets throttles on CI when lots of builds are running.
    //
    // let (protoc_bin, include_dir) =
    //     protoc_prebuilt::init("29.3").expect("failed to initialize protoc");

    // This version downloads a version of protoc bundled in the library,
    // which looks like just another Rust dependency, so shouldn't cause API rate limit issues.
    let protoc_bin = protoc_bin_vendored::protoc_bin_path().expect("can't find protoc for this OS");
    let include_dir =
        protoc_bin_vendored::include_path().expect("can't find protobuf includes for this OS");

    // Set the path to the bin so `prost_build` can find it.
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
