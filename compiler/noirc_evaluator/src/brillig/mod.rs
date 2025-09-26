//! The `brillig` module contains all logic necessary for noirc's Brillig-gen pass
//! for generating Brillig bytecode from [Ssa].
//!
//! # Usage
//!
//! Brillig generation is performed by calling the [Ssa::to_brillig] method.
//! All compiled Brillig artifacts will be returned as the [Brillig] context structure.
pub(crate) mod brillig_gen;
pub mod brillig_ir;

use acvm::FieldElement;
use brillig_gen::brillig_block::BrilligBlock;
use brillig_gen::constant_allocation::ConstantAllocation;
use brillig_gen::{brillig_fn::FunctionContext, brillig_globals::BrilligGlobals};
use brillig_ir::BrilligContext;
use brillig_ir::{artifact::LabelType, brillig_variable::BrilligVariable, registers::GlobalSpace};
use noirc_errors::call_stack::CallStackHelper;

use self::brillig_ir::{
    artifact::{BrilligArtifact, Label},
    procedures::compile_procedure,
};

use crate::brillig::brillig_ir::LayoutConfig;
use crate::ssa::{
    ir::{
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        types::NumericType,
        value::ValueId,
    },
    ssa_gen::Ssa,
};
use rustc_hash::FxHashMap as HashMap;
use std::{borrow::Cow, collections::BTreeSet};

pub use self::brillig_ir::procedures::ProcedureId;

/// Options that affect Brillig code generation.
#[derive(Default, Clone, Debug)]
pub struct BrilligOptions {
    pub enable_debug_trace: bool,
    pub enable_debug_assertions: bool,
    pub enable_array_copy_counter: bool,
    pub layout: LayoutConfig,
}

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default, Clone)]
pub struct Brillig {
    /// Maps SSA function labels to their brillig artifact
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    pub call_stacks: CallStackHelper,
    globals: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    globals_memory_size: HashMap<FunctionId, usize>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(
        &mut self,
        func: &Function,
        options: &BrilligOptions,
        globals: &HashMap<ValueId, BrilligVariable>,
        hoisted_global_constants: &HashMap<(FieldElement, NumericType), BrilligVariable>,
        is_entry_point: bool,
    ) {
        let obj = self.convert_ssa_function(
            func,
            options,
            globals,
            hoisted_global_constants,
            is_entry_point,
        );
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }

    /// Finds a brillig artifact by its label
    pub(crate) fn find_by_label(
        &self,
        function_label: Label,
        options: &BrilligOptions,
    ) -> Option<Cow<BrilligArtifact<FieldElement>>> {
        match function_label.label_type {
            LabelType::Function(function_id, _) => {
                self.ssa_function_to_brillig.get(&function_id).map(Cow::Borrowed)
            }
            // Procedures are compiled as needed
            LabelType::Procedure(procedure_id) => {
                Some(Cow::Owned(compile_procedure(procedure_id, options)))
            }
            LabelType::GlobalInit(function_id) => self.globals.get(&function_id).map(Cow::Borrowed),
            _ => unreachable!("ICE: Expected a function or procedure label"),
        }
    }

    /// Converting an SSA function into Brillig bytecode.
    pub(crate) fn convert_ssa_function(
        &mut self,
        func: &Function,
        options: &BrilligOptions,
        globals: &HashMap<ValueId, BrilligVariable>,
        hoisted_global_constants: &HashMap<(FieldElement, NumericType), BrilligVariable>,
        is_entry_point: bool,
    ) -> BrilligArtifact<FieldElement> {
        let mut brillig_context = BrilligContext::new(func.name(), options);

        let mut function_context = FunctionContext::new(func, is_entry_point);

        brillig_context.enter_context(Label::function(func.id()));

        brillig_context.call_check_max_stack_depth_procedure();

        for block in function_context.blocks.clone() {
            BrilligBlock::compile(
                &mut function_context,
                &mut brillig_context,
                block,
                &func.dfg,
                &mut self.call_stacks,
                globals,
                hoisted_global_constants,
            );
        }

        brillig_context.artifact()
    }
}

impl std::ops::Index<FunctionId> for Brillig {
    type Output = BrilligArtifact<FieldElement>;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.ssa_function_to_brillig[&id]
    }
}

impl Ssa {
    /// Compile Brillig functions and ACIR functions reachable from them
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn to_brillig(&self, options: &BrilligOptions) -> Brillig {
        let used_globals_map = self.used_globals_in_functions();

        // Collect all the function ids that are reachable from brillig
        // That means all the functions marked as brillig and ACIR functions called by them
        let brillig_reachable_function_ids = self
            .functions
            .iter()
            .filter_map(|(id, func)| func.runtime().is_brillig().then_some(*id))
            .collect::<BTreeSet<_>>();

        let mut brillig = Brillig::default();

        if brillig_reachable_function_ids.is_empty() {
            return brillig;
        }

        let mut brillig_globals = BrilligGlobals::new(self, used_globals_map, self.main_id);

        // SSA Globals are computed once at compile time and shared across all functions,
        // thus we can just fetch globals from the main function.
        // This same globals graph will then be used to declare Brillig globals for the respective entry points.
        let globals = (*self.functions[&self.main_id].dfg.globals).clone();
        let globals_dfg = DataFlowGraph::from(globals);
        brillig_globals.declare_globals(&globals_dfg, &mut brillig, options);

        for brillig_function_id in brillig_reachable_function_ids {
            let empty_allocations = HashMap::default();
            let empty_const_allocations = HashMap::default();
            let (globals_allocations, hoisted_constant_allocations) = brillig_globals
                .get_brillig_globals(brillig_function_id)
                .unwrap_or((&empty_allocations, &empty_const_allocations));

            let func = &self.functions[&brillig_function_id];
            let is_entry_point = brillig_globals.entry_points().contains_key(&brillig_function_id);

            brillig.compile(
                func,
                options,
                globals_allocations,
                hoisted_constant_allocations,
                is_entry_point,
            );
        }

        brillig
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        brillig::{BrilligOptions, brillig_ir::LayoutConfig},
        ssa::ssa_gen::Ssa,
    };

    #[test]
    fn same_program_has_same_bytecode_with_different_memory_layouts() {
        let src = r#"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v7 = make_array [u32 1, u32 2, u32 3, u32 4, u32 5] : [u32; 5]
            v8 = allocate -> &mut [u32; 5]
            store v7 at v8
            v9 = lt v0, u32 5
            jmp b1(u32 0)
          b1(v1: u32):
            v11 = lt v1, u32 5
            jmpif v11 then: b2, else: b3
          b2():
            v15 = load v8 -> [u32; 5]
            constrain v9 == u1 1, "Index out of bounds"
            v16 = array_set v15, index v0, value v1
            store v16 at v8
            v17 = unchecked_add v1, u32 1
            jmp b1(v17)
          b3():
            v12 = load v8 -> [u32; 5]
            constrain v9 == u1 1, "Index out of bounds"
            v14 = array_get v12, index v0 -> u32
            constrain v14 == u32 4
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();

        let brillig_default_mem_layout = ssa.to_brillig(&BrilligOptions::default());

        let layout = LayoutConfig::new(4096, 64);
        let options = BrilligOptions { layout, ..Default::default() };
        let brillig_4096_stack_frame = ssa.to_brillig(&options);

        let byte_code_default =
            &brillig_default_mem_layout.ssa_function_to_brillig[&ssa.main_id].byte_code;
        let byte_code_4096_stack_frame =
            &brillig_4096_stack_frame.ssa_function_to_brillig[&ssa.main_id].byte_code;
        // We could assert on the byte code vectors directly but if this test were to fail we want to see the exact opcodes that differ
        for (opcode_default, opcode_4096) in
            byte_code_default.iter().zip(byte_code_4096_stack_frame)
        {
            assert_eq!(opcode_default, opcode_4096);
        }
    }
}
