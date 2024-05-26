#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

// Arbitrary Circuit Intermediate Representation

pub mod circuit;
pub mod native_types;

pub use acir_field;
pub use acir_field::{AcirField, FieldElement};
pub use brillig;
pub use circuit::black_box_functions::BlackBoxFunc;

#[cfg(test)]
mod reflection {
    //! Getting test failures? You've probably changed the ACIR serialization format.
    //!
    //! These tests generate C++ deserializers for [`ACIR bytecode`][super::circuit::Circuit]
    //! and the [`WitnessMap`] structs. These get checked against the C++ files committed to the `codegen` folder
    //! to see if changes have been to the serialization format. These are almost always a breaking change!
    //!
    //! If you want to make a breaking change to the ACIR serialization format, then just comment out the assertions
    //! that the file hashes must match and rerun the tests. This will overwrite the `codegen` folder with the new
    //! logic. Make sure to uncomment these lines afterwards and to commit the changes to the `codegen` folder.

    use std::{
        fs::File,
        io::Write,
        path::{Path, PathBuf},
    };

    use acir_field::FieldElement;
    use brillig::{
        BinaryFieldOp, BinaryIntOp, BlackBoxOp, HeapValueType, Opcode as BrilligOpcode,
        ValueOrArray,
    };
    use serde_reflection::{Tracer, TracerConfig};

    use crate::{
        circuit::{
            brillig::{BrilligInputs, BrilligOutputs},
            directives::Directive,
            opcodes::{BlackBoxFuncCall, BlockType},
            AssertionPayload, Circuit, ExpressionOrMemory, ExpressionWidth, Opcode, OpcodeLocation,
            Program,
        },
        native_types::{Witness, WitnessMap, WitnessStack},
    };

    #[test]
    fn serde_acir_cpp_codegen() {
        let path = PathBuf::from("./codegen/acir.cpp");

        let old_hash = if path.is_file() {
            let old_source = std::fs::read(&path).unwrap();
            Some(fxhash::hash64(&old_source))
        } else {
            None
        };

        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<BlockType>().unwrap();
        tracer.trace_simple_type::<Program<FieldElement>>().unwrap();
        tracer.trace_simple_type::<Circuit<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ExpressionWidth>().unwrap();
        tracer.trace_simple_type::<Opcode<FieldElement>>().unwrap();
        tracer.trace_simple_type::<OpcodeLocation>().unwrap();
        tracer.trace_simple_type::<BinaryFieldOp>().unwrap();
        tracer.trace_simple_type::<BlackBoxFuncCall>().unwrap();
        tracer.trace_simple_type::<BrilligInputs<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BrilligOutputs>().unwrap();
        tracer.trace_simple_type::<BrilligOpcode<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BinaryIntOp>().unwrap();
        tracer.trace_simple_type::<BlackBoxOp>().unwrap();
        tracer.trace_simple_type::<Directive<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ValueOrArray>().unwrap();
        tracer.trace_simple_type::<HeapValueType>().unwrap();
        tracer.trace_simple_type::<AssertionPayload<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ExpressionOrMemory<FieldElement>>().unwrap();

        let registry = tracer.registry().unwrap();

        // Create C++ class definitions.
        let mut source = Vec::new();
        let config = serde_generate::CodeGeneratorConfig::new("Program".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, &registry).unwrap();

        // Comment this out to write updated C++ code to file.
        if let Some(old_hash) = old_hash {
            let new_hash = fxhash::hash64(&source);
            assert_eq!(new_hash, old_hash, "Serialization format has changed");
        }

        write_to_file(&source, &path);
    }

    #[test]
    fn serde_witness_map_cpp_codegen() {
        let path = PathBuf::from("./codegen/witness.cpp");

        let old_hash = if path.is_file() {
            let old_source = std::fs::read(&path).unwrap();
            Some(fxhash::hash64(&old_source))
        } else {
            None
        };

        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<Witness>().unwrap();
        tracer.trace_simple_type::<WitnessMap<FieldElement>>().unwrap();
        tracer.trace_simple_type::<WitnessStack<FieldElement>>().unwrap();

        let registry = tracer.registry().unwrap();

        // Create C++ class definitions.
        let mut source = Vec::new();
        let config = serde_generate::CodeGeneratorConfig::new("WitnessStack".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, &registry).unwrap();

        // Comment this out to write updated C++ code to file.
        if let Some(old_hash) = old_hash {
            let new_hash = fxhash::hash64(&source);
            assert_eq!(new_hash, old_hash, "Serialization format has changed");
        }

        write_to_file(&source, &path);
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
