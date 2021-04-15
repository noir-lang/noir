use acir::{circuit::gate::GadgetCall, native_types::Witness};
use noir_field::FieldElement;
use std::collections::BTreeMap;

pub struct GadgetCaller;

impl GadgetCaller {
    pub fn solve_gadget_call<F: FieldElement>(
        _initial_witness: &mut BTreeMap<Witness, F>,
        gadget_call: &GadgetCall,
    ) -> Result<(), acir::OPCODE> {
        // XXX: arkworks currently does not implement any of the ACIR opcodes
        // except for arithmetic
        Err(gadget_call.name)
    }
}
