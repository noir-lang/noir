use acvm::acir::native_types::Witness;
use acvm::acir::{circuit::opcodes::Opcode as AcirOpcode, native_types::Expression};
use iter_extended::btree_map;
use noirc_abi::{Abi, AbiDistinctness, AbiType, AbiVisibility};
use noirc_frontend::monomorphization::ast::{Definition, Program};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) mod parse_abi;

use crate::{
    errors::{RuntimeError, RuntimeErrorKind},
    ssa::ssa_gen::IrGenerator,
};

///! This module will create the Abi for a given program.

#[derive(Default)]
pub(crate) struct Evaluator {
    // Why is this not u64?
    //
    // At the moment, wasm32 is being used in the default backend
    // so it is safer to use a u32, at least until clang is changed
    // to compile wasm64.
    //
    // XXX: Barretenberg, reserves the first index to have value 0.
    // When we increment, we do not use this index at all.
    // This means that every constraint system at the moment, will either need
    // to decrease each index by 1, or create a dummy witness.
    //
    // We ideally want to not have this and have Barretenberg apply the
    // following transformation to the witness index : f(i) = i + 1
    current_witness_index: u32,
    // This is the number of witnesses indices used when
    // creating the private/public inputs of the ABI.
    num_witnesses_abi_len: usize,
    param_witnesses: BTreeMap<String, Vec<Witness>>,
    // This is the list of witness indices which are linked to public parameters.
    // Witnesses below `num_witnesses_abi_len` and not included in this set
    // correspond to private parameters and must not be made public.
    public_parameters: BTreeSet<Witness>,
    // The witness indices for return values are not guaranteed to be contiguous
    // and increasing as for `public_parameters`. We then use a `Vec` rather
    // than a `BTreeSet` to preserve this order for the ABI.
    return_values: Vec<Witness>,
    // If true, indicates that the resulting ACIR should enforce that all inputs and outputs are
    // comprised of unique witness indices by having extra constraints if necessary.
    return_is_distinct: bool,

    opcodes: Vec<AcirOpcode>,
}

impl Evaluator {
    // Returns true if the `witness_index` appears in the program's input parameters.
    fn is_abi_input(&self, witness_index: Witness) -> bool {
        witness_index.as_usize() <= self.num_witnesses_abi_len
    }

    // Returns true if the `witness_index`
    // was created in the ABI as a private input.
    //
    // Note: This method is used so that we don't convert private
    // ABI inputs into public outputs.
    fn is_private_abi_input(&self, witness_index: Witness) -> bool {
        // If the `witness_index` is more than the `num_witnesses_abi_len`
        // then it was created after the ABI was processed and is therefore
        // an intermediate variable.

        let is_public_input = self.public_parameters.contains(&witness_index);

        self.is_abi_input(witness_index) && !is_public_input
    }

    // True if the main function return has the `distinct` keyword and this particular witness
    // index has already occurred elsewhere in the abi's inputs and outputs.
    fn should_proxy_witness_for_abi_output(&self, witness_index: Witness) -> bool {
        self.return_is_distinct
            && (self.is_abi_input(witness_index) || self.return_values.contains(&witness_index))
    }

    // Creates a new Witness index
    fn add_witness_to_cs(&mut self) -> Witness {
        self.current_witness_index += 1;
        Witness(self.current_witness_index)
    }

    pub fn current_witness_index(&self) -> u32 {
        self.current_witness_index
    }

    pub fn push_opcode(&mut self, gate: AcirOpcode) {
        self.opcodes.push(gate);
    }

    /// Generates the ABI for the given program
    ///
    /// The return types in the ABI could have their Witnesses
    /// computed before the ACIR is computed, but then we would
    /// need to backtrace the ACIR to overwrite the final
    pub fn generate_abi(program: &Program) -> Abi {
        // First find out if the Program used the distinct keyword
        // indicating that the output should all have distinct witness indices.
        //
        // This will be used when processing the (only) return instruction
        // in the SSA IR.
        let should_output_have_distinct_witnesses =
            program.return_distinctness == AbiDistinctness::Distinct;

        todo!()
    }
}
