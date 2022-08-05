use acvm::acir::{circuit::gate::GadgetCall, native_types::Witness, OPCODE};
use acvm::FieldElement;
use std::collections::BTreeMap;

pub struct GadgetCaller;

impl GadgetCaller {
    pub fn solve_gadget_call(
        _initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gadget_call: &GadgetCall,
    ) -> Result<(), OPCODE> {
        // XXX: arkworks currently does not implement any of the ACIR opcodes
        // except for arithmetic
        Err(gadget_call.name)
    }
}
