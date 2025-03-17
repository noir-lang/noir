//! The Abstract Circuit Intermediate Representation (ACIR)
//!
//! The purpose of ACIR is to make the link between a generic proving system, such
//! as Aztec's Barretenberg, and a frontend, such as Noir, which describes user
//! specific computations.
//!
//! More precisely, Noir is a programming language for zero-knowledge proofs (ZKP)
//! which allows users to write programs in an intuitive way using a high-level
//! language close to Rust syntax. Noir is able to generate a proof of execution of
//! a Noir program, using an external proving system. However, proving systems use
//! specific low-level constrain-based languages. Similarly, frontends have their
//! own internal representation in order to represent user programs.
//!
//! The goal of ACIR is to provide a generic open-source intermediate
//! representation close to proving system 'languages', but agnostic to a specific
//! proving system, that can be used both by proving system as well as a target for
//! frontends. So, at the end of the day, an ACIR program is just another
//! representation of a program, dedicated to proving systems.
//!
//! ## Abstract Circuit Intermediate Representation
//! ACIR stands for abstract circuit intermediate representation:
//! - **abstract circuit**: circuits are a simple computation model where basic
//!     computation units, named gates, are connected with wires. Data flows
//!     through the wires while gates compute output wires based on their input.
//!     More formally, they are directed acyclic graphs (DAG) where the vertices
//!     are the gates and the edges are the wires. Due to the immutability nature
//!     of the wires (their value does not change during an execution), they are
//!     well suited for describing computations for ZKPs. Furthermore, we do not
//!     lose any expressiveness when using a circuit as it is well known that any
//!     bounded computation can be translated into an arithmetic circuit (i.e a
//!     circuit with only addition and multiplication gates).
//!     The term abstract here simply means that we do not refer to an actual physical
//!     circuit (such as an electronic circuit). Furthermore, we will not exactly use
//!     the circuit model, but another model even better suited to ZKPs, the constraint
//!     model (see below).
//! - **intermediate representation**: The ACIR representation is intermediate
//!   because it lies between a frontend and its proving system. ACIR bytecode makes
//!   the link between noir compiler output and the proving system backend input.
//!
//! ## The constraint model
//!
//! The first step for generating a proof that a specific program was executed, is
//! to execute this program. Since the proving system is going to handle ACIR
//! programs, we need in fact to execute an ACIR program, using the user-supplied
//! inputs.
//!
//! In ACIR terminology, the gates are called opcodes and the wires are called
//! partial witnesses. However, instead of connecting the opcodes together through
//! wires, we create constraints: an opcode constraints together a set of wires.
//! This constraint model trivially supersedes the circuit model. For instance, an
//! addition gate `output_wire = input_wire_1 + input_wire_2` can be expressed with
//! the following arithmetic constraint:
//! `output_wire - (input_wire_1 + input_wire_2) = 0`
//!
//! ## Solving
//!
//! Because of these constraints, executing an ACIR program is called solving the
//! witnesses. From the witnesses representing the inputs of the program, whose
//! values are supplied by the user, we find out what the other witnesses should be
//! by executing/solving the constraints one-by-one in the order they were defined.
//!
//! For instance, if `input_wire_1` and `input_wire_2` values are supplied as `3` and
//! `8`, then we can solve the opcode
//! `output_wire - (input_wire_1 + input_wire_2) = 0` by saying that `output_wire` is
//! `11`.
//!
//! In summary, the workflow is the following:
//! 1. user program -> (compilation) ACIR, a list of opcodes which constrain
//!     (partial) witnesses
//! 2. user inputs + ACIR -> (execution/solving) assign values to all the
//!     (partial) witnesses
//! 3. witness assignment + ACIR -> (proving system) proof
//!
//! Although the ordering of opcode does not matter in theory, since a system of
//! equations is not dependent on its ordering, in practice it matters a lot for the
//! solving (i.e the performance of the execution). ACIR opcodes **must be ordered**
//! so that each opcode can be resolved one after the other.
//!
//! The values of the witnesses lie in the scalar field of the proving system. We
//! will refer to it as `FieldElement` or ACIR field. The proving system needs the
//! values of all the partial witnesses and all the constraints in order to generate
//! a proof.
//!
//! _Remark_: The value of a partial witness is unique and fixed throughout a program
//!     execution, although in some rare cases, multiple values are possible for a
//!     same execution and witness (when there are several valid solutions to the
//!     constraints). Having multiple possible values for a witness may indicate that
//!     the circuit is not safe.
//!
//! _Remark_: Why do we use the term partial witnesses? It is because the proving
//!     system may create other constraints and witnesses (especially with
//!     `BlackBoxFuncCall`, see below). A proof refers to a full witness assignments
//!     and their constraints. ACIR opcodes and their partial witnesses are still an
//!     intermediate representation before getting the full list of constraints and
//!     witnesses. For the sake of simplicity, we will refer to witness instead of
//!     partial witness from now on.

#![cfg_attr(not(test), forbid(unsafe_code))] // `std::env::set_var` is used in tests.
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

pub mod circuit;
pub mod native_types;
mod proto;

pub use acir_field;
pub use acir_field::{AcirField, FieldElement};
pub use brillig;
pub use circuit::black_box_functions::BlackBoxFunc;
pub use circuit::opcodes::InvalidInputBitSize;

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
        BinaryFieldOp, BinaryIntOp, BitSize, BlackBoxOp, HeapValueType, IntegerBitSize,
        MemoryAddress, Opcode as BrilligOpcode, ValueOrArray,
    };
    use serde_reflection::{Tracer, TracerConfig};

    use crate::{
        circuit::{
            AssertionPayload, Circuit, ExpressionOrMemory, ExpressionWidth, Opcode, OpcodeLocation,
            Program,
            brillig::{BrilligInputs, BrilligOutputs},
            opcodes::{BlackBoxFuncCall, BlockType, ConstantOrWitnessEnum, FunctionInput},
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
        tracer.trace_simple_type::<ConstantOrWitnessEnum<FieldElement>>().unwrap();
        tracer.trace_simple_type::<FunctionInput<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BlackBoxFuncCall<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BrilligInputs<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BrilligOutputs>().unwrap();
        tracer.trace_simple_type::<BrilligOpcode<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BinaryIntOp>().unwrap();
        tracer.trace_simple_type::<BlackBoxOp>().unwrap();
        tracer.trace_simple_type::<ValueOrArray>().unwrap();
        tracer.trace_simple_type::<HeapValueType>().unwrap();
        tracer.trace_simple_type::<AssertionPayload<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ExpressionOrMemory<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BitSize>().unwrap();
        tracer.trace_simple_type::<IntegerBitSize>().unwrap();
        tracer.trace_simple_type::<MemoryAddress>().unwrap();

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
