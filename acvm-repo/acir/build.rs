use std::{path::Path, process::Command};

fn main() {
    let (protoc_bin, include_dir) =
        protoc_prebuilt::init("29.3").expect("failed to initialize protoc");

    unsafe {
        std::env::set_var("PROTOC", &protoc_bin);
    }

    // Schemas we use on the Noir side. `prost` generates bindings for imported schemas transitively.
    let noir_protos = ["./src/proto/program.proto", "./src/proto/acir/witness.proto"];

    // Schemas we use in `aztec-packages`, ie. Barretenberg. For these we use `protoc`
    // which doesn't generate source for imports, we have to list everything we need.
    let bb_protos = [
        // DTOs for `Program`, which works with the types in `acir.cpp`, pared down
        // so Barretenberg can ignore the Brillig part.
        // If we don't generate C++ bindings here then we can add this to the Rust
        // protos above to make to make sure it compiles.
        "./src/proto/acir/program.proto",
        // DTOs for the `WitnessStack`, which work with the types in `witness.cpp`
        "./src/proto/acir/witness.proto",
        "./src/proto/acir/circuit.proto",
        "./src/proto/acir/native.proto",
    ];

    let includes = [Path::new("./src/proto"), include_dir.as_path()];

    // Compile Rust sources into the OUT_DIR, e.g. target/debug/build/acir-611425dff1e0c70c/out/acvm.program.rs
    prost_build::compile_protos(&noir_protos, &includes).expect("failed to compile .proto schemas");

    // Compile C++ sources as well; the `serde` based codegen happens in tests, but here we have all the info already,
    // and if the source files change then `prost` will have emitted the signals `cargo` to run the build again.
    proto_cpp_codegen(&protoc_bin, "./codegen", &bb_protos, &includes)
        .expect("failed to generate C++ bindings");
}

/// Compile C++ bindings for our protobuf schemas, to be shared with `aztec-packages`
/// similar to how the `serde_acir_cpp_codegen` test produced `acir.cpp`.
///
/// Unlike `serde`, however, we don't have to error out if the bindings changed,
/// because this is backwards compatible. We just have to make sure that if the
/// `serde` bindings also changed, then we will update the mapping on the C++
/// side as well in order to make use of the new types/fields.
///
/// Alternatively we can do this as part of the `aztec-packages` build.
fn proto_cpp_codegen(
    protoc_bin: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    protos: &[impl AsRef<Path>],
    includes: &[impl AsRef<Path>],
) -> std::io::Result<()> {
    let mut cmd = Command::new(protoc_bin.as_ref());
    cmd.arg("--cpp_out").arg(out_dir.as_ref());
    for i in includes {
        cmd.arg("-I").arg(i.as_ref());
    }
    for p in protos {
        cmd.arg(p.as_ref());
    }
    let output = match cmd.output() {
        Err(e) => {
            return Err(std::io::Error::other(format!("failed to invoke protoc: {}", e)));
        }
        Ok(output) => output,
    };
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "protoc failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}
