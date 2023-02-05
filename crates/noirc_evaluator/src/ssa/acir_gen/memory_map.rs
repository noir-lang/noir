use crate::{
    ssa::{
        acir_gen::InternalVar,
        context::SsaContext,
        mem::{ArrayId, MemArray},
    },
    Evaluator,
};
use acvm::acir::native_types::{Expression, Witness};
use std::collections::HashMap;

// maps memory address to expression
#[derive(Default)]
pub struct MemoryMap {
    inner: HashMap<u32, InternalVar>,
}

impl MemoryMap {
    //Map the outputs into the array
    pub(crate) fn map_array(&mut self, a: ArrayId, outputs: &[Witness], ctx: &SsaContext) {
        let array = &ctx.mem[a];
        let address = array.adr;
        for i in 0..array.len {
            let var = if i < outputs.len() as u32 {
                InternalVar::from(outputs[i as usize])
            } else {
                InternalVar::from(Expression::zero())
            };
            self.inner.insert(address + i, var);
        }
    }

    //Load array values into InternalVars
    //If create_witness is true, we create witnesses for values that do not have witness
    pub(crate) fn load_array(
        &mut self,
        array: &MemArray,
        create_witness: bool,
        evaluator: &mut Evaluator,
    ) -> Vec<InternalVar> {
        (0..array.len)
            .map(|i| {
                let address = array.adr + i;
                match self.inner.get_mut(&address) {
                    Some(memory) => {
                        // TODO this is creating a `Witness` for a constant expression
                        // TODO Since that function only returns `None` when
                        // TODO the expression is a constant.
                        //
                        // TODO If this is not a bug, then we should pass a flag to
                        // TODO `cached_witness` to tell it when we want to create a witness
                        // TODO for a constant expression
                        if create_witness && memory.cached_witness().is_none() {
                            let w =
                                evaluator.create_intermediate_variable(memory.expression().clone());
                            *self.inner.get_mut(&address).unwrap().cached_witness_mut() = Some(w);
                        }
                        self.inner[&address].clone()
                    }
                    None => array.values[i as usize].clone(),
                }
            })
            .collect()
    }

    pub(crate) fn internal_var(&self, key: &u32) -> Option<&InternalVar> {
        self.inner.get(key)
    }
    pub(crate) fn insert(&mut self, key: u32, value: InternalVar) -> Option<InternalVar> {
        self.inner.insert(key, value)
    }
}
