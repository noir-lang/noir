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

#[cfg(test)]
mod tests {
    use acvm::{
        FieldElement,
        acir::brillig::{BitSize, IntegerBitSize, Opcode},
    };

    use crate::{
        brillig::{
            BrilligOptions, GlobalSpace, LabelType, Ssa, brillig_gen::gen_brillig_for,
            brillig_ir::artifact::BrilligParameter,
        },
        ssa::ir::map::Id,
    };

    use super::ConstantAllocation;

    #[test]
    fn single_entry_point_globals() {
        // This matches `entry_points_different_globals` except with a Brillig main
        let src = "
        g0 = Field 80

        brillig(inline) fn main f0 {
          b0(v1: Field, v2: Field):
            v4 = call f1(v1) -> Field
            constrain v4 == Field 2
            v6 = call f2(v1) -> Field
            constrain v6 == Field 2
            return
        }
        brillig(inline) fn entry_point_no_globals f1 {
          b0(v1: Field):
            v3 = add v1, Field 1
            v4 = add v3, Field 1
            return v4
        }
        brillig(inline) fn entry_point_globals f2 {
          b0(v1: Field):
            v2 = add v1, Field 80
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let options = BrilligOptions::default();
        let brillig = ssa.to_brillig(&options);
        dbg!(&brillig.globals);
        let args = vec![BrilligParameter::SingleAddr(254)];
        let entry =
            gen_brillig_for(&ssa.functions[&Id::test_new(2)], args, &brillig, &options, true)
                .unwrap();
        println!("{:#?}", entry.byte_code);
        // TODO: tests on linearization should be brought to the module where we have the Brillig entry point code gen
    }

    // TODO: we could make these execution unit tests
    // We have similar integration tests but would be good to have there as well
    #[test]
    fn entry_points_different_globals() {
        let src = "
        g0 = Field 80

        acir(inline) fn main f0 {
          b0(v1: Field, v2: Field):
            v4 = call f1(v1) -> Field
            constrain v4 == Field 2
            v6 = call f2(v1) -> Field
            constrain v6 == Field 2
            return
        }
        brillig(inline) fn entry_point_no_globals f1 {
          b0(v1: Field):
            v3 = add v1, Field 1
            v4 = add v3, Field 1
            return v4
        }
        brillig(inline) fn entry_point_globals f2 {
          b0(v1: Field):
            v2 = add v1, Field 80
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let options = BrilligOptions::default();
        let brillig = ssa.to_brillig(&options);
        dbg!(&brillig.globals);
        let args = vec![BrilligParameter::SingleAddr(254)];
        let entry =
            gen_brillig_for(&ssa.functions[&Id::test_new(2)], args, &brillig, &options, true)
                .unwrap();
        println!("{:#?}", entry.byte_code);

        // assert_eq!(
        //     brillig.globals.len(),
        //     2,
        //     "Should have a globals artifact associated with each entry point"
        // );
        // for (func_id, mut artifact) in brillig.globals {
        //     let labels = artifact.take_labels();
        //     // When entering a context two labels are created.
        //     // One is a context label and another is a section label.
        //     assert_eq!(labels.len(), 2);
        //     for (label, position) in labels {
        //         assert_eq!(label.label_type, LabelType::GlobalInit(func_id));
        //         assert_eq!(position, 0);
        //     }
        //     if func_id.to_u32() == 1 {
        //         assert_eq!(
        //             artifact.byte_code.len(),
        //             1,
        //             "Expected just a `Return`, but got more than a single opcode"
        //         );
        //         assert!(matches!(&artifact.byte_code[0], Opcode::Return));
        //     } else if func_id.to_u32() == 2 {
        //         assert_eq!(
        //             artifact.byte_code.len(),
        //             2,
        //             "Expected enough opcodes to initialize the globals"
        //         );
        //         let Opcode::Const { destination, bit_size, value } = &artifact.byte_code[0] else {
        //             panic!("First opcode is expected to be `Const`");
        //         };
        //         assert_eq!(
        //             destination.unwrap_direct(),
        //             GlobalSpace::start_with_layout(&options.layout)
        //         );
        //         assert!(matches!(bit_size, BitSize::Field));
        //         assert_eq!(*value, FieldElement::from(2u128));
        //         assert!(matches!(&artifact.byte_code[1], Opcode::Return));
        //     } else {
        //         panic!("Unexpected function id: {func_id}");
        //     }
        // }
    }

    #[test]
    fn entry_point_nested_globals() {
        let src = "
        g0 = Field 1
        g1 = make_array [Field 1, Field 1] : [Field; 2]
        g2 = Field 0
        g3 = make_array [Field 0, Field 0] : [Field; 2]
        g4 = make_array [g1, g3] : [[Field; 2]; 2]

        acir(inline) fn main f0 {
          b0(v5: Field, v6: Field):
              v8 = call f1(v5) -> Field
              constrain v8 == Field 2
              call f2(v5, v6)
              v12 = call f1(v5) -> Field
              constrain v12 == Field 2
              call f3(v5, v6)
              v15 = call f1(v5) -> Field
              constrain v15 == Field 2
              return
        }
        brillig(inline) fn entry_point_no_globals f1 {
          b0(v5: Field):
              v6 = add v5, Field 1
              v7 = add v6, Field 1
              return v7
        }
        brillig(inline) fn check_acc_entry_point f2 {
          b0(v5: Field, v6: Field):
              v8 = allocate -> &mut Field
              store Field 0 at v8
              jmp b1(u32 0)
          b1(v7: u32):
              v11 = lt v7, u32 2
              jmpif v11 then: b3, else: b2
          b2():
              v12 = load v8 -> Field
              v13 = eq v12, Field 0
              constrain v13 == u1 0
              v15 = eq v5, v6
              constrain v15 == u1 0
              v16 = add v5, Field 1
              v17 = add v16, Field 1
              constrain v17 == Field 2
              return
          b3():
              v19 = array_get g4, index v7 -> [Field; 2]
              v20 = load v8 -> Field
              v21 = array_get v19, index u32 0 -> Field
              v22 = add v20, v21
              v24 = array_get v19, index u32 1 -> Field
              v25 = add v22, v24
              store v25 at v8
              v26 = unchecked_add v7, u32 1
              jmp b1(v26)
        }
        brillig(inline) fn entry_point_inner_func_globals f3 {
          b0(v5: Field, v6: Field):
              call f4(v5, v6)
              return
        }
        brillig(inline) fn non_entry_point_wrapper f4 {
          b0(v5: Field, v6: Field):
              call f2(v5, v6)
              call f2(v5, v6)
              return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        // Need to run SSA pass that sets up Brillig array gets
        let ssa = ssa.brillig_array_get_and_set();

        let options = BrilligOptions::default();
        let brillig = ssa.to_brillig(&options);

        // assert_eq!(
        //     brillig.globals.len(),
        //     3,
        //     "Should have a globals artifact associated with each entry point"
        // );
        // for (func_id, mut artifact) in brillig.globals {
        //     let labels = artifact.take_labels();
        //     // When entering a context two labels are created.
        //     // One is a context label and another is a section label.
        //     assert_eq!(labels.len(), 2);
        //     for (label, position) in labels {
        //         assert_eq!(label.label_type, LabelType::GlobalInit(func_id));
        //         assert_eq!(position, 0);
        //     }
        //     if func_id.to_u32() == 1 {
        //         assert_eq!(
        //             artifact.byte_code.len(),
        //             2,
        //             "Expected enough opcodes to initialize the globals"
        //         );
        //         let Opcode::Const { destination, bit_size, value } = &artifact.byte_code[0] else {
        //             panic!("First opcode is expected to be `Const`");
        //         };
        //         assert_eq!(
        //             destination.unwrap_direct(),
        //             GlobalSpace::start_with_layout(&options.layout)
        //         );
        //         assert!(matches!(bit_size, BitSize::Field));
        //         assert_eq!(*value, FieldElement::from(1u128));
        //         assert!(matches!(&artifact.byte_code[1], Opcode::Return));
        //     } else if func_id.to_u32() == 2 || func_id.to_u32() == 3 {
        //         // We want the entry point which uses globals (f2) and the entry point which calls f2 function internally (f3 through f4)
        //         // to have the same globals initialized.
        //         assert_eq!(
        //             artifact.byte_code.len(),
        //             30,
        //             "Expected enough opcodes to initialize the globals"
        //         );
        //         let globals_max_memory = brillig
        //             .globals_memory_size
        //             .get(&func_id)
        //             .copied()
        //             .expect("Should have globals memory size");
        //         assert_eq!(globals_max_memory, 7);
        //     } else {
        //         panic!("Unexpected function id: {func_id}");
        //     }
        // }
    }

    #[test]
    fn hoist_shared_constants() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field):
            call f1(v0, v1)
            return
        }
        brillig(inline) predicate_pure fn entry_point f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            v4 = add v2, Field 1
            v6 = eq v4, Field 5
            constrain v6 == u1 0
            call f2(v0, v1)
            return
        }
        brillig(inline) predicate_pure fn inner_func f2 {
          b0(v0: Field, v1: Field):
            v3 = eq v0, Field 20
            constrain v3 == u1 0
            v5 = add v0, v1
            v7 = add v5, Field 10
            v9 = add v7, Field 1
            v11 = eq v9, Field 20
            constrain v11 == u1 0
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        // Show that the constants in each function have different SSA value IDs
        for (func_id, function) in &ssa.functions {
            let constant_allocation = ConstantAllocation::from_function(function);
            let mut constants = constant_allocation.get_constants().into_iter().collect::<Vec<_>>();
            // We want to order the constants by ID
            constants.sort();
            if func_id.to_u32() == 1 {
                assert_eq!(constants.len(), 3);
                let one = function.dfg.get_numeric_constant(constants[0]).unwrap();
                assert_eq!(one, FieldElement::from(1u128));
                let five = function.dfg.get_numeric_constant(constants[1]).unwrap();
                assert_eq!(five, FieldElement::from(5u128));
                let zero = function.dfg.get_numeric_constant(constants[2]).unwrap();
                assert_eq!(zero, FieldElement::from(0u128));
            } else if func_id.to_u32() == 2 {
                assert_eq!(constants.len(), 4);
                let twenty = function.dfg.get_numeric_constant(constants[0]).unwrap();
                assert_eq!(twenty, FieldElement::from(20u128));
                let zero = function.dfg.get_numeric_constant(constants[1]).unwrap();
                assert_eq!(zero, FieldElement::from(0u128));
                let ten = function.dfg.get_numeric_constant(constants[2]).unwrap();
                assert_eq!(ten, FieldElement::from(10u128));
                let one = function.dfg.get_numeric_constant(constants[3]).unwrap();
                assert_eq!(one, FieldElement::from(1u128));
            }
        }

        let options = BrilligOptions::default();
        let brillig = ssa.to_brillig(&options);

        // assert_eq!(brillig.globals.len(), 1, "Should have a single entry point");
        // for (func_id, artifact) in brillig.globals {
        //     assert_eq!(func_id.to_u32(), 1);
        //     assert_eq!(
        //         artifact.byte_code.len(),
        //         3,
        //         "Expected enough opcodes to initialize the hoisted constants"
        //     );
        //     let Opcode::Const { destination, bit_size, value } = &artifact.byte_code[0] else {
        //         panic!("First opcode is expected to be `Const`");
        //     };
        //     assert_eq!(
        //         destination.unwrap_direct(),
        //         GlobalSpace::start_with_layout(&options.layout)
        //     );
        //     assert!(matches!(bit_size, BitSize::Integer(IntegerBitSize::U1)));
        //     assert_eq!(*value, FieldElement::from(0u128));

        //     let Opcode::Const { destination, bit_size, value } = &artifact.byte_code[1] else {
        //         panic!("First opcode is expected to be `Const`");
        //     };
        //     assert_eq!(
        //         destination.unwrap_direct(),
        //         GlobalSpace::start_with_layout(&options.layout) + 1
        //     );
        //     assert!(matches!(bit_size, BitSize::Field));
        //     assert_eq!(*value, FieldElement::from(1u128));

        //     assert!(matches!(&artifact.byte_code[2], Opcode::Return));
        // }
    }

    #[test]
    fn do_not_hoist_shared_constants_different_entry_points() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field):
            call f1(v0, v1)
            call f2(v0, v1)
            return
        }
        brillig(inline) predicate_pure fn entry_point f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            v4 = add v2, Field 1
            v6 = eq v4, Field 5
            constrain v6 == u1 0
            return
        }
        brillig(inline) predicate_pure fn entry_point_two f2 {
          b0(v0: Field, v1: Field):
            v3 = eq v0, Field 20
            constrain v3 == u1 0
            v5 = add v0, v1
            v7 = add v5, Field 10
            v9 = add v7, Field 1
            v10 = eq v9, Field 20
            constrain v10 == u1 0
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let brillig = ssa.to_brillig(&BrilligOptions::default());

        // assert_eq!(
        //     brillig.globals.len(),
        //     2,
        //     "Should have a globals artifact associated with each entry point"
        // );
        // for (func_id, mut artifact) in brillig.globals {
        //     let labels = artifact.take_labels();
        //     // When entering a context two labels are created.
        //     // One is a context label and another is a section label.
        //     assert_eq!(labels.len(), 2);
        //     for (label, position) in labels {
        //         assert_eq!(label.label_type, LabelType::GlobalInit(func_id));
        //         assert_eq!(position, 0);
        //     }
        //     assert_eq!(
        //         artifact.byte_code.len(),
        //         1,
        //         "Expected enough opcodes to initialize the hoisted constants"
        //     );
        //     assert!(matches!(&artifact.byte_code[0], Opcode::Return));
        // }
    }
}
