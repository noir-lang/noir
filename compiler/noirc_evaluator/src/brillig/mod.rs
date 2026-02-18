//! The `brillig` module contains all logic necessary for noirc's Brillig-gen pass
//! for generating Brillig bytecode from [Ssa].
//!
//! # Usage
//!
//! Brillig generation is performed by calling the [Ssa::to_brillig] method.
//! All compiled Brillig artifacts will be returned as the [Brillig] context structure.
mod brillig_check;
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

/// Converts a u32 value to usize, panicking if the conversion fails.
pub(crate) fn assert_usize(value: u32) -> usize {
    value.try_into().expect("Failed conversion from u32 to usize")
}

/// Converts a usize value to u32, panicking if the conversion fails.
pub(crate) fn assert_u32(value: usize) -> u32 {
    value.try_into().expect("Failed conversion from usize to u32")
}

/// Options that affect Brillig code generation.
#[derive(Clone, Debug, Default)]
pub struct BrilligOptions {
    pub enable_debug_trace: bool,
    pub enable_debug_assertions: bool,
    pub enable_array_copy_counter: bool,
    pub show_opcode_advisories: bool,
    pub layout: LayoutConfig,
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
        let mut function_context =
            FunctionContext::new(func, is_entry_point, options.layout.max_stack_frame_size());

        let mut brillig_context =
            BrilligContext::new(func.name(), options, function_context.spill_support);

        brillig_context.enter_context(Label::function(func.id()));

        brillig_context.call_check_max_stack_depth_procedure();

        // Only emit spill prologue placeholders when the function may need spilling.
        if function_context.spill_support {
            brillig_context.emit_unresolved_spill_prologue();
        }

        for block in function_context.reverse_post_order().collect::<Vec<_>>() {
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

        // Resolve: overwrite placeholder NOPs with real allocation if spilling occurred
        if function_context.did_spill {
            brillig_context.resolve_spill_prologue(function_context.max_spill_offset);
        }

        if options.show_opcode_advisories {
            let opcode_advisories =
                brillig_check::opcode_advisories(func, &function_context, &brillig_context);

            brillig_check::show_opcode_advisories(&opcode_advisories, brillig_context.artifact());
        }

        let mut artifact = brillig_context.into_artifact();
        artifact.spill_support = function_context.spill_support;
        artifact
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
                            // free_memory_pointer = stack_start + max_stack_size
                            // (heap starts right after the stack, no global spill region)
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

    #[test]
    #[should_panic = "ICE: `BlackBoxFunc::RecursiveAggregation` calls are disallowed in Brillig"]
    fn disallows_compiling_recursive_aggregation_instructions() {
        let src = r#"
            brillig(inline) predicate_pure fn main f0 {
              b0(v0: u32):
                v1 = make_array [Field 0] : [Field; 1]
                v2 = make_array [Field 0] : [Field; 1]
                v3 = make_array [Field 0] : [Field; 1]
                call recursive_aggregation(v1, v2, v3, Field 0, u32 0)
                return
            }
        "#;

        let ssa = Ssa::from_str(src).unwrap();

        let _ = ssa.to_brillig(&BrilligOptions::default());
    }
}

#[cfg(test)]
mod spill_runtime {
    use acvm::FieldElement;

    use crate::{
        brillig::{
            BrilligOptions,
            brillig_gen::gen_brillig_for,
            brillig_ir::{
                LayoutConfig, artifact::BrilligParameter, registers::MAX_SCRATCH_SPACE,
                tests::create_and_run_vm,
            },
        },
        ssa::ssa_gen::Ssa,
    };

    /// Compile an SSA program with the given layout, then run the VM and return outputs.
    fn compile_and_run_with_layout(
        src: &str,
        calldata: Vec<FieldElement>,
        layout: LayoutConfig,
        args: Vec<BrilligParameter>,
    ) -> Vec<FieldElement> {
        let ssa = Ssa::from_str(src).unwrap();
        let options = BrilligOptions { layout, ..Default::default() };
        let brillig = ssa.to_brillig(&options);
        let main = ssa.main();
        let generated = gen_brillig_for(main, args, &brillig, &options).unwrap();

        let (vm, return_data_offset, return_data_size) =
            create_and_run_vm(calldata, &generated.byte_code);
        vm.get_memory()[return_data_offset..return_data_offset + return_data_size]
            .iter()
            .map(|v| v.to_field())
            .collect()
    }

    /// Minimal arithmetic spill test. Frame=6 gives 4 usable slots (start_offset=2).
    /// Two params (v0, v1) use 2 slots, leaving 2 free. Computing v2, v3 fills the
    /// frame. Computing v4 requires a constant temp that pushes past 4, forcing a spill.
    ///
    /// v0=3, v1=5:
    /// v2 = 3+5 = 8, v3 = 3+2 = 5, v4 = 5+3 = 8, v5 = 8+5 = 13, v6 = 13+8 = 21
    #[test]
    fn spill_arithmetic_correctness() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v0, u32 2
            v4 = unchecked_add v1, u32 3
            v5 = unchecked_add v2, v3
            v6 = unchecked_add v5, v4
            return v6
        }
        ";

        let calldata = vec![FieldElement::from(3u64), FieldElement::from(5u64)];
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        // Default layout: no spilling
        let correct = compile_and_run_with_layout(
            src,
            calldata.clone(),
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(21u64)]);

        // Small frame: forces spilling
        let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(src, calldata, layout, args);
        assert_eq!(result, vec![FieldElement::from(21u64)]);
    }

    /// Conditional control flow with spilling in the then-branch.
    /// Frame=7 gives 5 usable slots. In b1, v0 plus the b3 successor param
    /// use 2 of 5 slots. Computing v2, v3, v4 fills to 5 (FULL). Computing
    /// v5 exceeds the frame and forces a spill.
    ///
    /// v0=3 (b1 path): v2=4, v3=5, v4=6, v5=3+4=7, v6=7+5=12, v7=12+6=18
    /// v0=100 (b2 path): v8=100-1=99
    #[test]
    fn spill_with_conditional_control_flow() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = lt v0, u32 10
            jmpif v1 then: b1, else: b2
          b1():
            v2 = unchecked_add v0, u32 1
            v3 = unchecked_add v0, u32 2
            v4 = unchecked_add v0, u32 3
            v5 = unchecked_add v0, v2
            v6 = unchecked_add v5, v3
            v7 = unchecked_add v6, v4
            jmp b3(v7)
          b2():
            v8 = unchecked_sub v0, u32 1
            jmp b3(v8)
          b3(v9: u32):
            return v9
        }
        ";

        let args = vec![BrilligParameter::SingleAddr(32)];

        // Default layout: no spilling
        let correct = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(3u64)],
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(18u64)]);
        let correct = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(100u64)],
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(99u64)]);

        // Small frame: spilling in b1
        let layout = LayoutConfig::new(7, 16, MAX_SCRATCH_SPACE);

        let result =
            compile_and_run_with_layout(src, vec![FieldElement::from(3u64)], layout, args.clone());
        assert_eq!(result, vec![FieldElement::from(18u64)]);

        // v0=100: b2 path → 99
        let layout = LayoutConfig::new(7, 16, MAX_SCRATCH_SPACE);
        let result =
            compile_and_run_with_layout(src, vec![FieldElement::from(100u64)], layout, args);
        assert_eq!(result, vec![FieldElement::from(99u64)]);
    }

    /// Caller/callee program where the caller has enough live values to trigger
    /// spilling. Frame=12 gives 10 usable slots. The caller builds up 8 live
    /// values ({v0, v1, v2, v3, v4, v5, v6, v7}), and computing v8 forces a
    /// spill. Values v2, v3 must survive the call (may reload from spill slots).
    /// The frame must also satisfy `stack_size + call_args < max_frame_size`.
    ///
    /// v0=3, v1=5:
    /// v2=8, v3=5, v4=8, v5=7, v6=10, v7=9
    /// v8 = v4+v5 = 15, v9 = v6+v7 = 19, v10 = 15+19 = 34
    /// v11 = helper(3,5) = 8, v12 = v2+v3 = 13, v13 = 13+34 = 47
    /// v14 = 47+8 = 55
    #[test]
    fn spill_with_nested_call() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v0, u32 2
            v4 = unchecked_add v1, u32 3
            v5 = unchecked_add v0, u32 4
            v6 = unchecked_add v1, u32 5
            v7 = unchecked_add v0, u32 6
            v8 = unchecked_add v4, v5
            v9 = unchecked_add v6, v7
            v10 = unchecked_add v8, v9
            v11 = call f1(v0, v1) -> u32
            v12 = unchecked_add v2, v3
            v13 = unchecked_add v12, v10
            v14 = unchecked_add v13, v11
            return v14
        }
        brillig(inline) fn helper f1 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            return v2
        }
        ";

        let calldata = vec![FieldElement::from(3u64), FieldElement::from(5u64)];
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        // Default layout: no spilling
        let correct = compile_and_run_with_layout(
            src,
            calldata.clone(),
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(55u64)]);

        // Small frame: forces spilling in caller, values survive across call
        let layout = LayoutConfig::new(12, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(src, calldata, layout, args);
        assert_eq!(result, vec![FieldElement::from(55u64)]);
    }

    /// Cross-block spill correctness test. Frame=6 gives 4 usable slots.
    /// In b0, v0 is used once early (in v2) then not again — making it the LRU
    /// victim when pressure exceeds the frame. Its register gets reused.
    /// In b1, v0 is live-in (not a block parameter) and must be reloaded from
    /// its spill slot.
    ///
    /// v0=3, v1=5:
    /// v2=8, v3=7, v4=8, v5=8+7=15, v6=15+8=23
    /// b1: v7 = 23+3 = 26
    #[test]
    fn cross_block_spill_stale_register() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v1, u32 2
            v4 = unchecked_add v1, u32 3
            v5 = unchecked_add v2, v3
            v6 = unchecked_add v5, v4
            jmp b1()
          b1():
            v7 = unchecked_add v6, v0
            return v7
        }
        ";

        let calldata = vec![FieldElement::from(3u64), FieldElement::from(5u64)];
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        // Default layout: no spilling
        let correct = compile_and_run_with_layout(
            src,
            calldata.clone(),
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(26u64)]);

        // Small frame: v0 spilled in b0, reloaded in b1
        let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(src, calldata, layout, args);
        assert_eq!(result, vec![FieldElement::from(26u64)]);
    }

    /// Demonstrates that pinned successor block params can exhaust the register
    /// frame. With frame=5 (3 usable slots), b0 defines v0 (own) + v1,v2,v3
    /// (b1's params). After spilling v0, the LRU is empty and the
    /// terminator can't reload v0 to copy it to b1's params.
    ///
    /// v0=3: v1=v2=v3=3, v4=6, v5=9 → expected [9]
    #[test]
    fn spill_many_successor_params() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(v0, v0, v0)
          b1(v1: u32, v2: u32, v3: u32):
            v4 = unchecked_add v1, v2
            v5 = unchecked_add v4, v3
            return v5
        }
        ";

        let calldata = vec![FieldElement::from(3u64)];
        let args = vec![BrilligParameter::SingleAddr(32)];

        // Default layout: works fine
        let correct = compile_and_run_with_layout(
            src,
            calldata.clone(),
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(9u64)]);

        // Tiny layout (frame=5, 3 usable slots): successor params can be spilled
        // and their values written directly to spill slots at the Jmp terminator.
        let tiny = LayoutConfig::new(5, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(src, calldata, tiny, args);
        assert_eq!(result, vec![FieldElement::from(9u64)]);
    }

    /// Cross-block spill in a loop. Frame=10 gives 8 usable slots. In b0, the
    /// immediate dominator also allocates b1's params (v5, v6), so baseline overhead is
    /// {v0, v1, v5, v6} = 4 of 8. Computing v2..v4 plus constants fills the
    /// frame, and v0 (LRU) gets spilled.
    ///
    /// In b2, v0 is reloaded from its spill slot. The spill slot must NOT be
    /// freed on reload, because subsequent loop iterations need to reload from
    /// the same slot.
    ///
    /// v0=3, v1=5:
    /// v2=8, v3=7, v4=8+7=15
    /// Loop 3 times: 15+3=18, 18+3=21, 21+3=24
    #[test]
    fn cross_block_spill_in_loop() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v1, u32 2
            v4 = unchecked_add v2, v3
            jmp b1(v4, u32 0)
          b1(v5: u32, v6: u32):
            v7 = lt v6, u32 3
            jmpif v7 then: b2, else: b3
          b2():
            v8 = unchecked_add v5, v0
            v9 = unchecked_add v6, u32 1
            jmp b1(v8, v9)
          b3():
            return v5
        }
        ";

        let calldata = vec![FieldElement::from(3u64), FieldElement::from(5u64)];
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        // Default layout: no spilling
        let correct = compile_and_run_with_layout(
            src,
            calldata.clone(),
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(24u64)]);

        // Small frame: v0 spilled in b0, reloaded each loop iteration
        let layout = LayoutConfig::new(10, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(src, calldata.clone(), layout, args.clone());
        assert_eq!(result, vec![FieldElement::from(24u64)]);

        // Regression: frame=8 previously corrupted loop counter
        let layout = LayoutConfig::new(8, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(src, calldata, layout, args);
        assert_eq!(result, vec![FieldElement::from(24u64)]);
    }

    /// Diamond control flow where b1 (then-branch) spills v0, which is
    /// a non-param live-in of b3 (the merge block). v0 is b0's param but
    /// NOT b3's param — it enters b3 as a live-in without being passed
    /// as a block argument. The merge block sees it as spilled and emits
    /// reload, but on the else path nobody wrote to the spill slot.
    ///
    /// RPO: b0, b2, b1, b3 — b1 (then) is compiled AFTER b2 (else).
    /// b1 spills v0 (untouched → LRU victim). b3 sees v0 as spilled.
    /// On b0→b2→b3 path, spill slot was never written → garbage.
    ///
    /// v0=3  (b1 path): v6=210, v8=210+3=213
    /// v0=100 (b2 path): v7=100, v8=100+100=200
    #[test]
    fn cross_block_spill_diamond() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = lt v0, u32 10
            jmpif v1 then: b1, else: b2
          b1():
            v2 = unchecked_add u32 10, u32 20
            v3 = unchecked_add u32 30, u32 40
            v4 = unchecked_add u32 50, u32 60
            v5 = unchecked_add v2, v3
            v6 = unchecked_add v5, v4
            jmp b3(v6)
          b2():
            jmp b3(v0)
          b3(v7: u32):
            v8 = unchecked_add v7, v0
            return v8
        }
        ";
        let args = vec![BrilligParameter::SingleAddr(32)];

        // Default layout: correct
        let correct = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(100u64)],
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(200u64)]);
        let correct = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(3u64)],
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(213u64)]);

        // Frame=6: b1 spills v0, b3 emits reload.
        let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(100u64)],
            layout,
            args.clone(),
        );
        assert_eq!(result, vec![FieldElement::from(200u64)]);
        let result =
            compile_and_run_with_layout(src, vec![FieldElement::from(3u64)], layout, args.clone());
        assert_eq!(result, vec![FieldElement::from(213u64)]);
    }

    /// Verify Jmp argument swaps work correctly through permanent spill slots.
    /// With v0=3, v1=7: swap → v2=v1=7, v3=v0=3, v4=v0*v1=21
    /// v5 = v2*10 + v3 + v4 = 70 + 3 + 21 = 94
    #[test]
    fn jmp_parallel_move_swap_with_spilling() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v4 = unchecked_mul v0, v1
            jmp b1(v1, v0, v4)
          b1(v2: u32, v3: u32, v5: u32):
            v6 = unchecked_mul v2, u32 10
            v7 = unchecked_add v6, v3
            v8 = unchecked_add v7, v5
            return v8
        }
        ";
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        // Default layout: correct
        let correct = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(3u64), FieldElement::from(7u64)],
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(94u64)]);

        // Small frame to force spill_support — all params go through spill slots
        let layout = LayoutConfig::new(6, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(3u64), FieldElement::from(7u64)],
            layout,
            args.clone(),
        );
        assert_eq!(result, vec![FieldElement::from(94u64)]);
    }
}
