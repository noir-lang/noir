use acvm::acir::circuit::Circuit;
use noirc_abi::Abi;
use noirc_driver::CompiledProgram;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramArtifact {
    pub noir_version: String,

    /// Hash of the [`Program`][noirc_frontend::monomorphization::ast::Program] from which this [`ProgramArtifact`]
    /// was compiled.
    ///
    /// Used to short-circuit compilation in the case of the source code not changing since the last compilation.
    pub hash: u64,

    pub abi: Abi,

    #[serde(
        serialize_with = "Circuit::serialize_circuit_base64",
        deserialize_with = "Circuit::deserialize_circuit_base64"
    )]
    pub bytecode: Circuit,
}

impl From<CompiledProgram> for ProgramArtifact {
    fn from(program: CompiledProgram) -> Self {
        ProgramArtifact {
            hash: program.hash,
            abi: program.abi,
            noir_version: program.noir_version,
            bytecode: program.circuit,
        }
    }
}
