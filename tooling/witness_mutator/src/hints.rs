//! Brillig call sites: where a compiled program's witness is free.
//!
//! Every witness the ACVM assigns is either an input, the output of an opcode that also constrains
//! it (`AssertZero`, black box, memory), or the output of a `BrilligCall`. Only the last kind is
//! unconstrained by construction, so a second witness of the whole program is a different choice of
//! Brillig outputs that still satisfies every other opcode.

use acir::{
    AcirField, FieldElement,
    circuit::{
        Opcode, Program,
        brillig::{BrilligBytecode, BrilligFunctionId, BrilligOutputs},
    },
    native_types::{Witness, WitnessMap},
};
use acvm::acir::brillig::{
    BitSize, HeapVector, IntegerBitSize, MemoryAddress, Opcode as BrilligOpcode,
};

/// One `BrilligCall` opcode, with the values its outputs took during honest execution.
#[derive(Clone, Debug)]
pub struct HintSite {
    /// Index into `Program::functions`.
    pub function_index: usize,
    /// Index into that function's `opcodes`; this is what identifies one call site among several
    /// calls to the same Brillig function.
    pub opcode_index: usize,
    /// Which Brillig function is called. Stdlib directives are deduplicated, so every
    /// `directive_invert` site shares one id.
    pub brillig_id: BrilligFunctionId,
    /// Output witnesses, flattened in the order the ACVM writes the call's return data.
    pub outputs: Vec<Witness>,
    /// The values `outputs` took during honest execution.
    pub honest: Vec<FieldElement>,
}

impl HintSite {
    pub fn label(&self) -> String {
        format!("f{}:op{}", self.function_index, self.opcode_index)
    }
}

/// Collect the Brillig call sites of `function_index` and read their honest output values.
pub fn hint_sites(
    program: &Program<FieldElement>,
    function_index: usize,
    honest_witness: &WitnessMap<FieldElement>,
) -> Vec<HintSite> {
    let mut sites = Vec::new();
    for (opcode_index, opcode) in program.functions[function_index].opcodes.iter().enumerate() {
        let Opcode::BrilligCall { id, outputs, .. } = opcode else {
            continue;
        };
        let witnesses: Vec<Witness> = outputs
            .iter()
            .flat_map(|output| match output {
                BrilligOutputs::Simple(witness) => vec![*witness],
                BrilligOutputs::Array(witnesses) => witnesses.clone(),
            })
            .collect();

        // A call under a false predicate has its outputs zeroed by the solver rather than taken
        // from the bytecode, so overriding it can never produce a different witness.
        let Some(honest) =
            witnesses.iter().map(|witness| honest_witness.get(witness).copied()).collect()
        else {
            continue;
        };

        sites.push(HintSite {
            function_index,
            opcode_index,
            brillig_id: *id,
            outputs: witnesses,
            honest,
        });
    }
    sites
}

/// Brillig bytecode that ignores its inputs and returns `values`.
///
/// This is how an output override is expressed without touching the solver: the modified program
/// keeps every constraint, and only the hint behind one call site changes.
fn constant_bytecode(values: &[FieldElement]) -> BrilligBytecode<FieldElement> {
    let count = values.len();
    let size_address = MemoryAddress::direct(count as u32 + 1);
    let pointer_address = MemoryAddress::direct(count as u32 + 2);

    let mut bytecode = vec![
        BrilligOpcode::Const {
            destination: size_address,
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::from(count),
        },
        BrilligOpcode::Const {
            destination: pointer_address,
            bit_size: BitSize::Integer(IntegerBitSize::U32),
            value: FieldElement::zero(),
        },
    ];
    for (index, value) in values.iter().enumerate() {
        bytecode.push(BrilligOpcode::Const {
            destination: MemoryAddress::direct(index as u32),
            bit_size: BitSize::Field,
            value: *value,
        });
    }
    bytecode.push(BrilligOpcode::Stop {
        return_data: HeapVector { pointer: pointer_address, size: size_address },
    });

    BrilligBytecode { bytecode, function_name: "witness_mutator_override".to_string() }
}

/// A copy of `program` in which `site` returns `values` instead of running its hint.
///
/// The overriding bytecode is appended as a new unconstrained function and only this one call site
/// is repointed at it, so other sites sharing the same `BrilligFunctionId` keep their real hint.
pub fn program_with_override(
    program: &Program<FieldElement>,
    site: &HintSite,
    values: &[FieldElement],
) -> Program<FieldElement> {
    let mut program = program.clone();
    let new_id = BrilligFunctionId::new(program.unconstrained_functions.len() as u32);
    program.unconstrained_functions.push(constant_bytecode(values));

    match &mut program.functions[site.function_index].opcodes[site.opcode_index] {
        Opcode::BrilligCall { id, .. } => *id = new_id,
        _ => unreachable!("hint site does not point at a BrilligCall"),
    }
    program
}
