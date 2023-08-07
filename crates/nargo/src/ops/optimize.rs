use acvm::{
    acir::circuit::{Circuit, OpcodeLabel},
    Backend,
};
use base64::Engine;
use iter_extended::vecmap;
use noirc_abi::Abi;
use noirc_driver::{CompiledProgram, FunctionType};
use noirc_errors::debug_info::DebugInfo;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::NargoError;

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizedProgram {
    /// The name of the contract.
    pub name: Option<String>,

    /// Each of the contract's functions are compiled into a separate program stored in this `Vec`.
    pub functions: Vec<OptimizedFunction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizedFunction {
    pub name: String,

    pub function_type: FunctionType,

    pub is_internal: bool,

    pub abi: Abi,

    #[serde(serialize_with = "serialize_circuit", deserialize_with = "deserialize_circuit")]
    pub bytecode: Circuit,

    #[serde(skip)]
    pub debug: DebugInfo,
}

// TODO: move these down into ACVM.
fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).unwrap();
    let encoded_b64 = base64::engine::general_purpose::STANDARD.encode(circuit_bytes);
    s.serialize_str(&encoded_b64)
}

fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let bytecode_b64: String = serde::Deserialize::deserialize(deserializer)?;
    let circuit_bytes = base64::engine::general_purpose::STANDARD.decode(bytecode_b64).unwrap();
    let circuit = Circuit::read(&*circuit_bytes).unwrap();
    Ok(circuit)
}

/// Apply backend specific optimizations to produce an [OptimizedProgram].
pub fn optimize_program<B: Backend>(
    backend: &B,
    program: CompiledProgram,
) -> Result<OptimizedProgram, NargoError> {
    let mut optimized_functions = Vec::new();
    for func in program.functions {
        let (optimized_bytecode, opcode_labels) =
            acvm::compiler::compile(func.bytecode, backend.np_language(), |opcode| {
                backend.supports_opcode(opcode)
            })
            .map_err(|_| NargoError::CompilationError)?;

        let opcode_ids = vecmap(opcode_labels, |label| match label {
            OpcodeLabel::Unresolved => {
                // TODO: Surface this error?
                unreachable!("Compiled circuit opcodes must resolve to some index")
            }
            OpcodeLabel::Resolved(index) => index as usize,
        });

        // TODO: Non-mutating API
        let mut new_debug = func.debug.clone();
        new_debug.update_acir(opcode_ids);

        let optimized_function = OptimizedFunction {
            name: func.name,
            function_type: func.function_type,
            is_internal: func.is_internal,
            abi: func.abi,
            bytecode: optimized_bytecode,
            debug: new_debug,
        };

        optimized_functions.push(optimized_function);
    }

    Ok(OptimizedProgram { name: program.name, functions: optimized_functions })
}
