#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

// Arbitrary Circuit Intermediate Representation

pub mod circuit;
pub mod native_types;

pub use acir_field;
pub use acir_field::FieldElement;
pub use brillig;
pub use circuit::black_box_functions::BlackBoxFunc;

#[cfg(test)]
mod reflection {
    use std::{
        fs::File,
        io::Write,
        path::{Path, PathBuf},
    };

    use brillig::{
        BinaryFieldOp, BinaryIntOp, BlackBoxOp, ForeignCallParam, ForeignCallResult,
        Opcode as BrilligOpcode, RegisterOrMemory,
    };
    use serde_reflection::{Tracer, TracerConfig};

    use crate::{
        circuit::{
            brillig::{BrilligInputs, BrilligOutputs},
            directives::Directive,
            opcodes::BlackBoxFuncCall,
            Circuit, Opcode, OpcodeLocation,
        },
        native_types::{Witness, WitnessMap},
    };

    #[test]
    fn serde_acir_cpp_codegen() {
        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<Circuit>().unwrap();
        tracer.trace_simple_type::<Opcode>().unwrap();
        tracer.trace_simple_type::<OpcodeLocation>().unwrap();
        tracer.trace_simple_type::<BinaryFieldOp>().unwrap();
        tracer.trace_simple_type::<BlackBoxFuncCall>().unwrap();
        tracer.trace_simple_type::<BrilligInputs>().unwrap();
        tracer.trace_simple_type::<BrilligOutputs>().unwrap();
        tracer.trace_simple_type::<BrilligOpcode>().unwrap();
        tracer.trace_simple_type::<BinaryIntOp>().unwrap();
        tracer.trace_simple_type::<BlackBoxOp>().unwrap();
        tracer.trace_simple_type::<Directive>().unwrap();
        tracer.trace_simple_type::<ForeignCallParam>().unwrap();
        tracer.trace_simple_type::<ForeignCallResult>().unwrap();
        tracer.trace_simple_type::<RegisterOrMemory>().unwrap();

        let registry = tracer.registry().unwrap();

        let data = serde_json::to_vec(&registry).unwrap();
        write_to_file(&data, &PathBuf::from("./codegen/acir.json"));

        // Create C++ class definitions.
        let mut source = Vec::new();
        let config = serde_generate::CodeGeneratorConfig::new("Circuit".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, &registry).unwrap();

        write_to_file(&source, &PathBuf::from("./codegen/acir.cpp"));
    }

    #[test]
    fn serde_witnessmap_cpp_codegen() {
        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<Witness>().unwrap();
        tracer.trace_simple_type::<WitnessMap>().unwrap();

        let registry = tracer.registry().unwrap();

        let data = serde_json::to_vec(&registry).unwrap();
        write_to_file(&data, &PathBuf::from("./codegen/witness.json"));

        // Create C++ class definitions.
        let mut source = Vec::new();
        let config = serde_generate::CodeGeneratorConfig::new("WitnessMap".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, &registry).unwrap();

        write_to_file(&source, &PathBuf::from("./codegen/witness.cpp"));
    }

    fn write_to_file(bytes: &[u8], path: &Path) -> String {
        let display = path.display();

        let parent_dir = path.parent().unwrap();
        if !parent_dir.is_dir() {
            std::fs::create_dir_all(parent_dir).unwrap();
        }

        let mut file = match File::create(path) {
            Err(why) => panic!("couldn't create {display}: {why}"),
            Ok(file) => file,
        };

        match file.write_all(bytes) {
            Err(why) => panic!("couldn't write to {display}: {why}"),
            Ok(_) => display.to_string(),
        }
    }
}
