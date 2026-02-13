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

    /// Compile an SSA program with a small stack frame that forces spilling, then run the VM.
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

    /// Frame size 12 gives 10 usable slots (start_offset=2 for spill base).
    /// With SPILL_MARGIN=128, spill_support=true for any program.
    fn spill_layout() -> LayoutConfig {
        LayoutConfig::new(12, 16, MAX_SCRATCH_SPACE)
    }

    /// Computes 10 independent values from two inputs, keeping all of them live until
    /// the final summation chain. With 10 usable slots, computing the 11th value
    /// forces at least one spill. Uses unchecked arithmetic to avoid overflow-check
    /// temporaries that would inflate register pressure unpredictably.
    ///
    /// v0=3, v1=5:
    /// v2  = 3+5 = 8,  v3  = 3+2 = 5,  v4  = 3+3 = 6,  v5  = 5+4 = 9,
    /// v6  = 5+5 = 10, v7  = 3+6 = 9,  v8  = 5+7 = 12, v9  = 3+8 = 11,
    /// v10 = 5+9 = 14, v11 = 3+10 = 13
    /// sum = 8+5+6+9+10+9+12+11+14+13 = 97
    #[test]
    fn spill_arithmetic_correctness() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v0, u32 2
            v4 = unchecked_add v0, u32 3
            v5 = unchecked_add v1, u32 4
            v6 = unchecked_add v1, u32 5
            v7 = unchecked_add v0, u32 6
            v8 = unchecked_add v1, u32 7
            v9 = unchecked_add v0, u32 8
            v10 = unchecked_add v1, u32 9
            v11 = unchecked_add v0, u32 10
            v12 = unchecked_add v2, v3
            v13 = unchecked_add v12, v4
            v14 = unchecked_add v13, v5
            v15 = unchecked_add v14, v6
            v16 = unchecked_add v15, v7
            v17 = unchecked_add v16, v8
            v18 = unchecked_add v17, v9
            v19 = unchecked_add v18, v10
            v20 = unchecked_add v19, v11
            return v20
        }
        ";

        let calldata = vec![FieldElement::from(3u64), FieldElement::from(5u64)];
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        let result = compile_and_run_with_layout(src, calldata, spill_layout(), args);
        assert_eq!(result, vec![FieldElement::from(97u64)]);
    }

    /// Control flow with spilling in at least one branch. Uses unchecked operations
    /// to keep register pressure predictable.
    ///
    /// b1 path (v0 < 10): computes 8 independent values from v0, then sums them.
    ///   Each v_i = v0 + i, for i in 1..8, plus v0 itself.
    ///   sum = v0 + (v0+1) + (v0+2) + ... + (v0+8) = 9*v0 + 36
    ///   For v0=3: 9*3 + 36 = 63
    ///
    /// b2 path (v0 >= 10): simple chain of subtractions.
    ///   v0 - 1 - 2 - 3 - 4 - 5 - 6 = v0 - 21
    ///   For v0=100: 100 - 21 = 79
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
            v5 = unchecked_add v0, u32 4
            v6 = unchecked_add v0, u32 5
            v7 = unchecked_add v0, u32 6
            v8 = unchecked_add v0, u32 7
            v9 = unchecked_add v0, u32 8
            v10 = unchecked_add v0, v2
            v11 = unchecked_add v10, v3
            v12 = unchecked_add v11, v4
            v13 = unchecked_add v12, v5
            v14 = unchecked_add v13, v6
            v15 = unchecked_add v14, v7
            v16 = unchecked_add v15, v8
            v17 = unchecked_add v16, v9
            jmp b3(v17)
          b2():
            v18 = unchecked_sub v0, u32 1
            v19 = unchecked_sub v18, u32 2
            v20 = unchecked_sub v19, u32 3
            v21 = unchecked_sub v20, u32 4
            v22 = unchecked_sub v21, u32 5
            v23 = unchecked_sub v22, u32 6
            jmp b3(v23)
          b3(v24: u32):
            return v24
        }
        ";

        let args = vec![BrilligParameter::SingleAddr(32)];

        // v0 = 3: b1 path → 9*3 + 36 = 63
        let result = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(3u64)],
            spill_layout(),
            args.clone(),
        );
        assert_eq!(result, vec![FieldElement::from(63u64)]);

        // v0 = 100: b2 path → 100 - 21 = 79
        let result = compile_and_run_with_layout(
            src,
            vec![FieldElement::from(100u64)],
            spill_layout(),
            args,
        );
        assert_eq!(result, vec![FieldElement::from(79u64)]);
    }

    /// Two-function program where both caller and callee have enough live values
    /// to trigger independent spilling. The caller consumes some values before
    /// the call to reduce register pressure at the call site (codegen_call
    /// allocates internal registers through the raw allocator, not the spill
    /// manager).
    ///
    /// Uses frame=16 (14 usable slots). Both functions compute 14 independent
    /// values from their inputs (16 live with v0,v1), forcing spills.
    ///
    /// v0=3, v1=5:
    /// Caller computes v2..v15 (same as callee), then consumes v8..v15 into
    /// a partial sum (v22=93) before calling helper. After the call, sums the
    /// remaining v2..v7 + v22 + helper_result.
    ///   partial_sum = 12+11+14+13+15+7+4+17 = 93
    ///   helper(3,5) = 8+5+6+9+10+9+12+11+14+13+15+7+4+17 = 140
    ///   remaining = 8+5+6+9+10+9 = 47
    ///   result = 47 + 93 + 140 = 280
    #[test]
    fn spill_with_nested_call() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v0, u32 2
            v4 = unchecked_add v0, u32 3
            v5 = unchecked_add v1, u32 4
            v6 = unchecked_add v1, u32 5
            v7 = unchecked_add v0, u32 6
            v8 = unchecked_add v1, u32 7
            v9 = unchecked_add v0, u32 8
            v10 = unchecked_add v1, u32 9
            v11 = unchecked_add v0, u32 10
            v12 = unchecked_add v0, u32 12
            v13 = unchecked_add v1, u32 2
            v14 = unchecked_add v0, u32 1
            v15 = unchecked_add v1, u32 12
            v16 = unchecked_add v8, v9
            v17 = unchecked_add v16, v10
            v18 = unchecked_add v17, v11
            v19 = unchecked_add v18, v12
            v20 = unchecked_add v19, v13
            v21 = unchecked_add v20, v14
            v22 = unchecked_add v21, v15
            v23 = call f1(v0, v1) -> u32
            v24 = unchecked_add v2, v3
            v25 = unchecked_add v24, v4
            v26 = unchecked_add v25, v5
            v27 = unchecked_add v26, v6
            v28 = unchecked_add v27, v7
            v29 = unchecked_add v28, v22
            v30 = unchecked_add v29, v23
            return v30
        }
        brillig(inline) fn helper f1 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v0, u32 2
            v4 = unchecked_add v0, u32 3
            v5 = unchecked_add v1, u32 4
            v6 = unchecked_add v1, u32 5
            v7 = unchecked_add v0, u32 6
            v8 = unchecked_add v1, u32 7
            v9 = unchecked_add v0, u32 8
            v10 = unchecked_add v1, u32 9
            v11 = unchecked_add v0, u32 10
            v12 = unchecked_add v0, u32 12
            v13 = unchecked_add v1, u32 2
            v14 = unchecked_add v0, u32 1
            v15 = unchecked_add v1, u32 12
            v16 = unchecked_add v2, v3
            v17 = unchecked_add v16, v4
            v18 = unchecked_add v17, v5
            v19 = unchecked_add v18, v6
            v20 = unchecked_add v19, v7
            v21 = unchecked_add v20, v8
            v22 = unchecked_add v21, v9
            v23 = unchecked_add v22, v10
            v24 = unchecked_add v23, v11
            v25 = unchecked_add v24, v12
            v26 = unchecked_add v25, v13
            v27 = unchecked_add v26, v14
            v28 = unchecked_add v27, v15
            return v28
        }
        ";

        // partial_sum(v8..v15) = 12+11+14+13+15+7+4+17 = 93
        // helper(3,5) = 8+5+6+9+10+9+12+11+14+13+15+7+4+17 = 140
        // remaining(v2..v7) = 8+5+6+9+10+9 = 47
        // result = 47 + 93 + 140 = 280
        let calldata = vec![FieldElement::from(3u64), FieldElement::from(5u64)];
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        let layout = LayoutConfig::new(16, 16, MAX_SCRATCH_SPACE);
        let result = compile_and_run_with_layout(src, calldata, layout, args);
        assert_eq!(result, vec![FieldElement::from(280u64)]);
    }

    /// Cross-block spill correctness test.
    ///
    /// In b0, v0 is used once early then never again — making it the LRU victim when
    /// register pressure exceeds the frame size. Its register gets reused for a later value.
    /// In b1, v0 is live-in (not a block parameter). The SpillManager persists across
    /// blocks, so b1 knows v0 is spilled and reloads it correctly.
    ///
    /// v0=3, v1=5:
    /// v2=8, v3=7, v4=8, v5=9, v6=10, v7=11, v8=12, v9=13, v10=14, v11=15
    /// sum(v2..v11) = 8+7+8+9+10+11+12+13+14+15 = 107
    /// correct v21 = 107 + v0(3) = 110
    #[test]
    fn cross_block_spill_stale_register() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v1, u32 2
            v4 = unchecked_add v1, u32 3
            v5 = unchecked_add v1, u32 4
            v6 = unchecked_add v1, u32 5
            v7 = unchecked_add v1, u32 6
            v8 = unchecked_add v1, u32 7
            v9 = unchecked_add v1, u32 8
            v10 = unchecked_add v1, u32 9
            v11 = unchecked_add v1, u32 10
            v12 = unchecked_add v2, v3
            v13 = unchecked_add v12, v4
            v14 = unchecked_add v13, v5
            v15 = unchecked_add v14, v6
            v16 = unchecked_add v15, v7
            v17 = unchecked_add v16, v8
            v18 = unchecked_add v17, v9
            v19 = unchecked_add v18, v10
            v20 = unchecked_add v19, v11
            jmp b1()
          b1():
            v21 = unchecked_add v20, v0
            return v21
        }
        ";

        let calldata = vec![FieldElement::from(3u64), FieldElement::from(5u64)];
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        // Default layout: no spilling -> correct result
        let correct = compile_and_run_with_layout(
            src,
            calldata.clone(),
            LayoutConfig::default(),
            args.clone(),
        );
        assert_eq!(correct, vec![FieldElement::from(110u64)]);

        // Small layout: spilling correctly persists across blocks
        let result = compile_and_run_with_layout(src, calldata, spill_layout(), args);
        assert_eq!(result, vec![FieldElement::from(110u64)]);
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

    /// Cross-block spill in a loop: a value spilled before the loop is used inside
    /// the loop body. The spill slot must not be reused (freed) on reload, because
    /// subsequent loop iterations need to reload from the same slot.
    ///
    /// v0=3, v1=5:
    /// b0: v0 used once, then lots of pressure causes it to be spilled.
    /// b1: loop header, v20 is the accumulator (starts at sum from b0).
    /// b2: loop body, reloads v0 and adds it to accumulator. Loops 3 times.
    /// Result: sum(v2..v11) + v0*3 = 107 + 9 = 116
    #[test]
    fn cross_block_spill_in_loop() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v1, u32 2
            v4 = unchecked_add v1, u32 3
            v5 = unchecked_add v1, u32 4
            v6 = unchecked_add v1, u32 5
            v7 = unchecked_add v1, u32 6
            v8 = unchecked_add v1, u32 7
            v9 = unchecked_add v1, u32 8
            v10 = unchecked_add v1, u32 9
            v11 = unchecked_add v1, u32 10
            v12 = unchecked_add v2, v3
            v13 = unchecked_add v12, v4
            v14 = unchecked_add v13, v5
            v15 = unchecked_add v14, v6
            v16 = unchecked_add v15, v7
            v17 = unchecked_add v16, v8
            v18 = unchecked_add v17, v9
            v19 = unchecked_add v18, v10
            v20 = unchecked_add v19, v11
            jmp b1(v20, u32 0)
          b1(v21: u32, v22: u32):
            v23 = lt v22, u32 3
            jmpif v23 then: b2, else: b3
          b2():
            v24 = unchecked_add v21, v0
            v25 = unchecked_add v22, u32 1
            jmp b1(v24, v25)
          b3():
            return v21
        }
        ";

        let calldata = vec![FieldElement::from(3u64), FieldElement::from(5u64)];
        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];

        // Default layout: no spilling -> correct result
        let correct = compile_and_run_with_layout(
            src,
            calldata.clone(),
            LayoutConfig::default(),
            args.clone(),
        );
        // 107 + 3*3 = 116
        assert_eq!(correct, vec![FieldElement::from(116u64)]);

        // Small layout: spilling with loop — spill slot must persist across iterations
        let result = compile_and_run_with_layout(src, calldata, spill_layout(), args);
        assert_eq!(result, vec![FieldElement::from(116u64)]);
    }
}
