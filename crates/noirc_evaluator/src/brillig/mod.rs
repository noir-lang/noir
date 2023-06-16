pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use self::{
    brillig_gen::{brillig_block::BrilligBlock, brillig_fn::FunctionContext},
    brillig_ir::{artifact::BrilligArtifact, BrilligContext},
};
use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId, RuntimeType},
        post_order::PostOrder,
    },
    ssa_gen::Ssa,
};
use std::collections::HashMap;

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA functions to their brillig opcode
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact>,
    /// Maps SSA functions to their entry block
    ssa_function_to_block: HashMap<FunctionId, BasicBlockId>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(&mut self, func: &Function) {
        let obj = self.convert_ssa_function(func);
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }

    pub(crate) fn convert_ssa_function(&self, func: &Function) -> BrilligArtifact {
        let mut reverse_post_order = Vec::new();
        reverse_post_order.extend_from_slice(PostOrder::with_function(func).as_slice());
        reverse_post_order.reverse();

        let mut function_context =
            FunctionContext { function_id: func.id(), ssa_value_to_register: HashMap::new() };

        let mut brillig_context = BrilligContext::new(self.function_labels());
        for block in reverse_post_order {
            BrilligBlock::compile(&mut function_context, &mut brillig_context, block, &func.dfg);
        }

        brillig_context.artifact()
    }

    /// Returns the function id concatenated with the block id
    pub(crate) fn function_block_label(&self, id: FunctionId) -> String {
        id.to_string() + "-" + &self.ssa_function_to_block[&id].to_string()
    }

    pub(crate) fn function_labels(
        &self,
    ) -> HashMap<crate::ssa_refactor::ir::map::Id<Function>, String> {
        let mut function_labels = HashMap::new();
        for func in self.ssa_function_to_block.keys() {
            function_labels.insert(*func, self.function_block_label(*func));
        }
        function_labels
    }
}

impl std::ops::Index<FunctionId> for Brillig {
    type Output = BrilligArtifact;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.ssa_function_to_brillig[&id]
    }
}

impl Ssa {
    /// Generate compilation artifacts for brillig functions
    pub(crate) fn to_brillig(&self) -> Brillig {
        let mut brillig = Brillig::default();
        for f in self.functions.values().filter(|func| func.runtime() == RuntimeType::Brillig) {
            brillig.ssa_function_to_block.insert(f.id(), f.entry_block());
        }

        for f in self.functions.values().filter(|func| func.runtime() == RuntimeType::Brillig) {
            let id = f.id();
            if id != self.main_id {
                brillig.compile(f);
            }
        }
        brillig
    }
}
