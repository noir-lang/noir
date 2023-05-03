//! This file holds the pass to convert from Noir's SSA IR to ACIR.
use super::ssa_gen::Ssa;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the currrent SSA IR.
struct Context {}

/// The output of the Acir-gen pass
pub struct Acir {}

impl Ssa {
    pub(crate) fn into_acir(self) -> Acir {
        let mut context = Context::new();
        context.convert_ssa(self)
    }
}

impl Context {
    fn new() -> Self {
        Self {}
    }

    fn convert_ssa(&mut self, _ssa: Ssa) -> Acir {
        todo!()
    }
}
