//! Codegen for converting SSA globals to Brillig bytecode.
use std::collections::BTreeSet;

use acvm::FieldElement;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::brillig_block::BrilligBlock;
use super::{BrilligVariable, FunctionContext, ValueId};
use crate::ssa::ssa_gen::Ssa;
use crate::{
    brillig::{
        Brillig, BrilligOptions, ConstantAllocation, DataFlowGraph, FunctionId, Label,
        brillig_ir::{BrilligContext, artifact::BrilligArtifact},
    },
    ssa::ir::types::NumericType,
};

/// Context structure for generating Brillig globals
/// it stores globals related data required for code generation of regular Brillig functions.
#[derive(Default)]
pub(crate) struct BrilligGlobals {
    /// Mapping of SSA value ids to Brillig allocations
    pub(crate) ssa_to_brillig: SsaToBrilligGlobals,
    /// Mapping of constant values hoisted to global memory
    pub(crate) hoisted_constants: HoistedConstantsToBrilligGlobals,
}

/// Mapping of SSA value ids to their Brillig allocations
pub(crate) type SsaToBrilligGlobals = HashMap<ValueId, BrilligVariable>;
/// Mapping of constant values shared across functions hoisted to the global memory space
pub(crate) type HoistedConstantsToBrilligGlobals =
    HashMap<(FieldElement, NumericType), BrilligVariable>;
/// Mapping of a constant value and the number of functions in which it occurs
pub(crate) type ConstantCounterMap = HashMap<(FieldElement, NumericType), usize>;

impl BrilligGlobals {
    pub(crate) fn new(
        ssa: &Ssa,
        used_globals_map: &HashMap<FunctionId, HashSet<ValueId>>,
        brillig: &mut Brillig,
        options: &BrilligOptions,
    ) -> Self {
        // Collect all function ids reachable from Brillig
        let brillig_reachable_function_ids: BTreeSet<FunctionId> = ssa
            .functions
            .iter()
            .filter_map(|(&id, func)| func.runtime().is_brillig().then_some(id))
            .collect();

        if brillig_reachable_function_ids.is_empty() {
            return Self::default();
        }

        // Aggregate used globals from reachable functions
        let mut used_globals = HashSet::default();
        for func_id in &brillig_reachable_function_ids {
            if let Some(globals) = used_globals_map.get(func_id) {
                used_globals.extend(globals.iter());
            }
        }

        // Hoist constants shared across functions
        let mut constant_counts = ConstantCounterMap::default();
        for &func_id in &brillig_reachable_function_ids {
            let func = &ssa.functions[&func_id];
            let constants = ConstantAllocation::from_function(func);
            for c in constants.get_constants() {
                let value = func.dfg.get_numeric_constant(c).unwrap();
                let typ = func.dfg.type_of_value(c).unwrap_numeric();
                if !func.dfg.is_global(c) {
                    *constant_counts.entry((value, typ)).or_insert(0) += 1;
                }
            }
        }

        let hoisted_constants: BTreeSet<_> = constant_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(const_key, _)| const_key)
            .collect();

        // SSA Globals are computed once at compile time and shared across all functions,
        // thus we can just fetch globals from the main function.
        // This same globals graph will then be used to declare Brillig globals for the respective entry points.
        let globals_dfg = DataFlowGraph::from((*ssa.functions[&ssa.main_id].dfg.globals).clone());
        // Convert SSA globals to Brillig globals
        let (artifact, ssa_to_brillig, globals_size, hoisted_constants) =
            brillig.convert_ssa_globals(options, &globals_dfg, &used_globals, &hoisted_constants);

        // Store in full Brillig artifact
        brillig.globals = artifact;
        brillig.globals_memory_size = globals_size;

        Self { ssa_to_brillig, hoisted_constants }
    }
}

/// A globals artifact containing all information necessary for utilizing
/// globals from SSA during Brillig code generation.
pub(crate) type BrilligGlobalsArtifact = (
    // The actual bytecode declaring globals and any metadata needing for linking
    BrilligArtifact<FieldElement>,
    // The SSA value -> Brillig global allocations
    // This will be used for fetching global values when compiling functions to Brillig.
    HashMap<ValueId, BrilligVariable>,
    // The size of the global memory
    usize,
    // Duplicate SSA constants local to a function -> Brillig global allocations
    HashMap<(FieldElement, NumericType), BrilligVariable>,
);

impl Brillig {
    pub(crate) fn convert_ssa_globals(
        &mut self,
        options: &BrilligOptions,
        globals_dfg: &DataFlowGraph,
        used_globals: &HashSet<ValueId>,
        hoisted_global_constants: &BTreeSet<(FieldElement, NumericType)>,
    ) -> BrilligGlobalsArtifact {
        let mut brillig_context = BrilligContext::new_for_global_init(options);
        // The global space does not have globals itself
        let empty_globals = HashMap::default();
        // We can use any ID here as this context is only going to be used for globals which does not differentiate
        // by functions and blocks. The only Label that should be used in the globals context is `Label::globals_init()`
        let mut function_context = FunctionContext::default();
        brillig_context.enter_context(Label::globals_init());

        let block_id = DataFlowGraph::default().make_block();
        let mut brillig_block = BrilligBlock {
            function_context: &mut function_context,
            block_id,
            brillig_context: &mut brillig_context,
            variables: Default::default(),
            last_uses: HashMap::default(),
            globals: &empty_globals,
            hoisted_global_constants: &HashMap::default(),
            building_globals: true,
        };

        let hoisted_global_constants = brillig_block.compile_globals(
            globals_dfg,
            used_globals,
            &mut self.call_stacks,
            hoisted_global_constants,
        );

        let globals_size = brillig_context.global_space_size();

        brillig_context.return_instruction();

        let artifact = brillig_context.artifact();
        (artifact, function_context.ssa_value_allocations, globals_size, hoisted_global_constants)
    }
}
