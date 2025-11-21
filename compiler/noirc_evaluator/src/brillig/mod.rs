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
    pub vm_version: acvm::brillig_vm::Version,
}

/// Context structure for the Brillig pass.
/// It stores Brillig-related data required for Brillig generation.
#[derive(Default, Clone)]
pub struct Brillig {
    /// Maps SSA function labels to their brillig artifact
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    call_stacks: CallStackHelper,
    /// Bytecode for globals for each Brillig entry point.
    globals: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    /// The size of the global space for each Brillig entry point.
    globals_memory_size: HashMap<FunctionId, usize>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts.
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

    /// Finds a brillig artifact by its label.
    pub(crate) fn find_by_label(
        &self,
        function_label: Label,
        options: &BrilligOptions,
        stack_start: usize,
    ) -> Option<Cow<BrilligArtifact<FieldElement>>> {
        match function_label.label_type {
            LabelType::Function(function_id, _) => {
                self.ssa_function_to_brillig.get(&function_id).map(Cow::Borrowed)
            }
            // Procedures are compiled as needed
            LabelType::Procedure(procedure_id) => {
                Some(Cow::Owned(compile_procedure(procedure_id, options, stack_start)))
            }
            LabelType::GlobalInit(function_id) => self.globals.get(&function_id).map(Cow::Borrowed),
            _ => unreachable!("ICE: Expected a function or procedure label"),
        }
    }

    /// Convert an SSA function into Brillig bytecode.
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
            BrilligBlock::compile_block(
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

    pub fn call_stacks(&self) -> &CallStackHelper {
        &self.call_stacks
    }
}

impl std::ops::Index<FunctionId> for Brillig {
    type Output = BrilligArtifact<FieldElement>;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.ssa_function_to_brillig[&id]
    }
}

impl Ssa {
    /// Compile Brillig functions and ACIR functions reachable from them.
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn to_brillig(&self, options: &BrilligOptions) -> Brillig {
        // Collect all the function ids that are reachable from Brillig.
        // That means all the functions marked as Brillig and ACIR functions called by them,
        // but we should already have monomorphized ACIR functions as a Brillig as well.
        let brillig_reachable_function_ids = self
            .functions
            .iter()
            .filter_map(|(id, func)| func.runtime().is_brillig().then_some(*id))
            .collect::<BTreeSet<_>>();

        let mut brillig = Brillig::default();

        if brillig_reachable_function_ids.is_empty() {
            return brillig;
        }

        // SSA Globals are computed once at compile time and shared across all functions,
        // thus we can just fetch globals from the main function.
        // This same globals graph will then be used to declare Brillig globals for the respective entry points.
        let globals = (*self.functions[&self.main_id].dfg.globals).clone();
        let globals_dfg = DataFlowGraph::from(globals);

        // Produce the globals Brillig bytecode and variable allocation for each entry point.
        let brillig_globals = BrilligGlobals::init(self, self.main_id).declare_globals(
            &globals_dfg,
            &mut brillig,
            options,
        );

        for brillig_function_id in brillig_reachable_function_ids {
            let (globals_allocations, hoisted_constant_allocations) =
                brillig_globals.get_brillig_globals(brillig_function_id);

            let func = &self.functions[&brillig_function_id];
            let is_entry_point = brillig_globals.is_entry_point(&brillig_function_id);

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
mod memory_layout {
    use acvm::{AcirField, acir::brillig::Opcode};

    use crate::{
        brillig::{
            BrilligOptions,
            brillig_gen::gen_brillig_for,
            brillig_ir::{
                LayoutConfig, ReservedRegisters,
                artifact::BrilligParameter,
                registers::{MAX_SCRATCH_SPACE, MAX_STACK_FRAME_SIZE, NUM_STACK_FRAMES},
            },
        },
        ssa::ssa_gen::Ssa,
    };

    fn assert_equivalent_bytecode<F: AcirField>(
        bytecode1: &[Opcode<F>],
        bytecode2: &[Opcode<F>],
        options1: &BrilligOptions,
        options2: &BrilligOptions,
    ) {
        assert_eq!(bytecode1.len(), bytecode2.len());

        // Offset where stack starts
        // This assumes the same SSA where we have a single global register and a single call data argument.
        let stack_start = ReservedRegisters::len() + MAX_SCRATCH_SPACE + 1 + 1;

        for (op1, op2) in bytecode1.iter().zip(bytecode2) {
            if op1 != op2 {
                match (op1, op2) {
                    // All opcodes are expected to be stable across memory layouts except for a few that rely
                    // on the max stack size and stack frame size (e.g., setting of the `free_memory_pointer` and stack bounds checks).
                    (
                        Opcode::Const { destination: dest1, bit_size: bits1, value: val1 },
                        Opcode::Const { destination: dest2, bit_size: bits2, value: val2 },
                    ) => {
                        assert_eq!(dest1, dest2);
                        assert_eq!(bits1, bits2);

                        if *dest1 == ReservedRegisters::free_memory_pointer() {
                            // free_memory_pointer depends on max_stack_size + stack_start
                            // This is where the heap begins.
                            let expected1 = options1.layout.max_stack_size() + stack_start;
                            let expected2 = options2.layout.max_stack_size() + stack_start;

                            assert_eq!(val1.to_u128(), expected1 as u128);
                            assert_eq!(val2.to_u128(), expected2 as u128);
                        } else {
                            // Stack depth bound check
                            // This is done by the CheckMaxStackDepth procedure
                            let bound1 = options1.layout.max_stack_size()
                                - options1.layout.max_stack_frame_size()
                                + stack_start;
                            let bound2 = options2.layout.max_stack_size()
                                - options2.layout.max_stack_frame_size()
                                + stack_start;

                            assert_eq!(val1.to_u128(), bound1 as u128);
                            assert_eq!(val2.to_u128(), bound2 as u128);
                        }
                    }
                    _ => panic!("Unexpected opcode difference {op1} != {op2}"),
                }
            }
        }
    }

    fn compiles_to_equivalent_bytecode(
        ssa: &Ssa,
        options1: BrilligOptions,
        options2: BrilligOptions,
    ) {
        let main = ssa.main();

        let brillig1 = ssa.to_brillig(&options1);
        let brillig2 = ssa.to_brillig(&options2);

        // SSA function level comparison
        let code1 = &brillig1.ssa_function_to_brillig[&ssa.main_id].byte_code;
        let code2 = &brillig2.ssa_function_to_brillig[&ssa.main_id].byte_code;
        assert_equivalent_bytecode(code1, code2, &options1, &options2);

        // Entry point level comparison
        let args = vec![BrilligParameter::SingleAddr(32)];
        let entry1 = gen_brillig_for(main, args.clone(), &brillig1, &options1).unwrap();
        let entry2 = gen_brillig_for(main, args, &brillig2, &options2).unwrap();

        assert_equivalent_bytecode(&entry1.byte_code, &entry2.byte_code, &options1, &options2);
    }

    const SRC: &str = r#"
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

    #[test]
    fn same_program_bytecode_with_different_stack_frame_size() {
        let ssa = Ssa::from_str(SRC).unwrap();

        let options1 = BrilligOptions::default();
        let layout = LayoutConfig::new(4096, NUM_STACK_FRAMES, MAX_SCRATCH_SPACE);
        let options2 = BrilligOptions { layout, ..Default::default() };

        compiles_to_equivalent_bytecode(&ssa, options1, options2);
    }

    #[test]
    fn same_program_with_different_num_stack_frames() {
        let ssa = Ssa::from_str(SRC).unwrap();

        let options1 = BrilligOptions::default();
        let layout = LayoutConfig::new(MAX_STACK_FRAME_SIZE, 32, MAX_SCRATCH_SPACE);
        let options2 = BrilligOptions { layout, ..Default::default() };

        compiles_to_equivalent_bytecode(&ssa, options1, options2);
    }
}
