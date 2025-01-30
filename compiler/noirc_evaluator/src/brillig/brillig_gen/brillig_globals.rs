use std::collections::{BTreeMap, BTreeSet};

use acvm::FieldElement;
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
    BrilligArtifact, BrilligBlock, BrilligVariable, Function, FunctionContext, Label, ValueId,
};
use crate::brillig::{called_functions_vec, Brillig, FunctionId, Instruction, Value};
use crate::{
    brillig::{brillig_ir::BrilligContext, ConstantAllocation, DataFlowGraph},
    ssa::ir::types::NumericType,
};

/// Context structure for generating Brillig globals
/// it stores globals related data required for code generation of regular Brillig functions.
#[derive(Default)]
pub(crate) struct BrilligGlobals {
    /// Both `used_globals` and `brillig_entry_points` need to be built
    /// from a function call graph.
    ///
    /// Maps a Brillig function to the globals used in that function.
    /// This includes all globals used in functions called internally.
    used_globals: HashMap<FunctionId, HashSet<ValueId>>,
    /// Maps a Brillig entry point to all functions called in that entry point.
    /// This includes any nested calls as well, as we want to be able to associate
    /// any Brillig function with the appropriate global allocations.
    brillig_entry_points: HashMap<FunctionId, HashSet<FunctionId>>,
    /// Maps a Brillig entry point to constants shared across the entry point and its nested calls.
    hoisted_global_constants: HashMap<FunctionId, ConstantCounterMap>,

    /// Maps an inner call to its Brillig entry point
    /// This is simply used to simplify fetching global allocations when compiling
    /// individual Brillig functions.
    inner_call_to_entry_point: HashMap<FunctionId, Vec<FunctionId>>,
    /// Final map that associates an entry point with its Brillig global allocations
    entry_point_globals_map: HashMap<FunctionId, SsaToBrilligGlobals>,
    /// Final map that associates an entry point with any local function constants
    /// that are shared and were hoisted to the global space.
    /// This map is kept separate from `entry_point_globals_map` to clearly distinguish
    /// the two types of globals.
    entry_point_hoisted_globals_map: HashMap<FunctionId, HoistedConstantsToBrilligGlobals>,
}

/// Mapping of SSA value ids to their Brillig allocations
pub(crate) type SsaToBrilligGlobals = HashMap<ValueId, BrilligVariable>;

pub(crate) type HoistedConstantsToBrilligGlobals =
    HashMap<(FieldElement, NumericType), BrilligVariable>;
/// Mapping of a constant value and the number of functions in which it occurs
pub(crate) type ConstantCounterMap = HashMap<(FieldElement, NumericType), usize>;

impl BrilligGlobals {
    pub(crate) fn new(
        functions: &BTreeMap<FunctionId, Function>,
        used_globals: HashMap<FunctionId, HashSet<ValueId>>,
        main_id: FunctionId,
    ) -> Self {
        let mut instance = Self { used_globals, ..Default::default() };

        instance.build_entry_point_info_from_call_graph(functions, main_id);

        instance
    }

    /// This methods collects information from the call graph of a Brillig entry point.
    /// - Update the used globals for a given entry point based upon its call graph.
    /// - Determine constants which are shared across multiple functions for possible hoisting
    ///   into the global memory space.
    /// - Build a simplified call graph of an entry point to all functions
    ///   possibly called from that entry point.
    fn build_entry_point_info_from_call_graph(
        &mut self,
        functions: &BTreeMap<FunctionId, Function>,
        main_id: FunctionId,
    ) {
        let acir_functions = functions.iter().filter(|(_, func)| func.runtime().is_acir());
        for (_, function) in acir_functions {
            for block_id in function.reachable_blocks() {
                for instruction_id in function.dfg[block_id].instructions() {
                    let instruction = &function.dfg[*instruction_id];
                    let Instruction::Call { func: func_id, arguments: _ } = instruction else {
                        continue;
                    };

                    let func_value = &function.dfg[*func_id];
                    let Value::Function(func_id) = func_value else { continue };

                    let called_function = &functions[func_id];
                    if called_function.runtime().is_acir() {
                        continue;
                    }

                    // We have now found a Brillig entry point.
                    // Let's recursively build a call graph to determine any functions
                    // whose parent is this entry point and any globals used in those internal calls.
                    self.brillig_entry_points.insert(*func_id, HashSet::default());
                    self.mark_entry_points_calls_recursive(
                        functions,
                        *func_id,
                        called_function,
                        im::HashSet::new(),
                    );
                }
            }
        }

        // If main has been marked as Brillig, it is itself an entry point.
        // Run the same analysis from above on main.
        let main_func = &functions[&main_id];
        if main_func.runtime().is_brillig() {
            self.brillig_entry_points.insert(main_id, HashSet::default());
            self.mark_entry_points_calls_recursive(
                functions,
                main_id,
                main_func,
                im::HashSet::new(),
            );
        }
    }

    /// Recursively mark any functions called in an entry point as well as
    /// any globals used in those functions.
    /// Using the information collected we can determine which globals
    /// an entry point must initialize.
    fn mark_entry_points_calls_recursive(
        &mut self,
        functions: &BTreeMap<FunctionId, Function>,
        entry_point: FunctionId,
        called_function: &Function,
        mut explored_functions: im::HashSet<FunctionId>,
    ) {
        if explored_functions.insert(called_function.id()).is_some() {
            return;
        }

        // We can potentially have multiple local constants with the same value and type
        let constants = ConstantAllocation::from_function(called_function);
        for constant in constants.get_constants() {
            let value = called_function.dfg.get_numeric_constant(constant);
            let value = value.unwrap();
            let typ = called_function.dfg.type_of_value(constant);
            if !called_function.dfg.is_global(constant) {
                self.hoisted_global_constants
                    .entry(entry_point)
                    .or_default()
                    .entry((value, typ.unwrap_numeric()))
                    .and_modify(|counter| *counter += 1)
                    .or_insert(1);
            }
        }

        let inner_calls = called_functions_vec(called_function).into_iter().collect::<HashSet<_>>();

        for inner_call in inner_calls {
            let inner_globals = self
                .used_globals
                .get(&inner_call)
                .expect("Should have a slot for each function")
                .clone();
            self.used_globals
                .get_mut(&entry_point)
                .expect("ICE: should have func")
                .extend(inner_globals);

            if let Some(inner_calls) = self.brillig_entry_points.get_mut(&entry_point) {
                inner_calls.insert(inner_call);
            }

            self.mark_entry_points_calls_recursive(
                functions,
                entry_point,
                &functions[&inner_call],
                explored_functions.clone(),
            );
        }
    }

    pub(crate) fn declare_globals(
        &mut self,
        globals_dfg: &DataFlowGraph,
        brillig: &mut Brillig,
        enable_debug_trace: bool,
    ) {
        // Map for fetching the correct entry point globals when compiling any function
        let mut inner_call_to_entry_point: HashMap<FunctionId, Vec<FunctionId>> =
            HashMap::default();

        let mut entry_point_globals_map = HashMap::default();
        let mut entry_point_hoisted_globals_map = HashMap::default();

        let mut all_hoisted_allocations = HashMap::default();
        // We only need to generate globals for entry points
        for (entry_point, entry_point_inner_calls) in self.brillig_entry_points.iter() {
            let entry_point = *entry_point;

            for inner_call in entry_point_inner_calls {
                inner_call_to_entry_point.entry(*inner_call).or_default().push(entry_point);
            }

            let used_globals = self.used_globals.remove(&entry_point).unwrap_or_default();
            // Select set of constants which can be hoisted from function's to the global memory space
            // for a given entry point.
            let hoisted_global_constants = self
                .hoisted_global_constants
                .remove(&entry_point)
                .unwrap_or_default()
                .iter()
                .filter_map(
                    |(&value, &num_occurrences)| {
                        if num_occurrences > 1 {
                            Some(value)
                        } else {
                            None
                        }
                    },
                )
                .collect();
            let (artifact, brillig_globals, globals_size, hoisted_global_constants) =
                convert_ssa_globals(
                    enable_debug_trace,
                    globals_dfg,
                    &used_globals,
                    &hoisted_global_constants,
                    entry_point,
                );

            all_hoisted_allocations.extend(hoisted_global_constants.clone());

            entry_point_globals_map.insert(entry_point, brillig_globals);
            entry_point_hoisted_globals_map.insert(entry_point, hoisted_global_constants);

            brillig.globals.insert(entry_point, artifact);
            brillig.globals_memory_size.insert(entry_point, globals_size);
        }

        self.inner_call_to_entry_point = inner_call_to_entry_point;
        self.entry_point_globals_map = entry_point_globals_map;
        self.entry_point_hoisted_globals_map = entry_point_hoisted_globals_map;
    }

    /// Fetch the global allocations that can possibly be accessed
    /// by any given Brillig function (non-entry point or entry point).
    /// The allocations available to a function are determined by its entry point.
    /// For a given function id input, this function will search for that function's
    /// entry point (or multiple entry points) and fetch the global allocations
    /// associated with those entry points.
    /// These allocations can then be used when compiling the Brillig function
    /// and resolving global variables.
    pub(crate) fn get_brillig_globals(
        &self,
        brillig_function_id: FunctionId,
    ) -> (SsaToBrilligGlobals, HoistedConstantsToBrilligGlobals) {
        let entry_points = self.inner_call_to_entry_point.get(&brillig_function_id);

        let mut globals_allocations = HashMap::default();
        let mut hoisted_constants_allocations = HashMap::default();
        if let Some(entry_points) = entry_points {
            // A Brillig function is used by multiple entry points. Fetch both globals allocations
            // in case one is used by the internal call.
            let entry_point_allocations = entry_points
                .iter()
                .flat_map(|entry_point| self.entry_point_globals_map.get(entry_point))
                .collect::<Vec<_>>();
            for map in entry_point_allocations {
                globals_allocations.extend(map);
            }

            // The global memory space is not shared across entry points.
            // Thus, if a call can be used by two entry points we should not use the hoisted constant allocations.
            if entry_points.len() == 1 {
                let hoisted_allocations = self.entry_point_hoisted_globals_map.get(&entry_points[0]).unwrap_or_else(|| panic!("ICE: Expected hoisted constant allocations to be set for function {brillig_function_id}"));
                hoisted_constants_allocations.extend(hoisted_allocations);
            }
        } else if let Some(globals) = self.entry_point_globals_map.get(&brillig_function_id) {
            // If there is no mapping from an inner call to an entry point, that means `brillig_function_id`
            // is itself an entry point and we can fetch the global allocations directly from `self.entry_point_globals_map`.
            globals_allocations.extend(globals);

            let hoisted_allocations = self.entry_point_hoisted_globals_map.get(&brillig_function_id).unwrap_or_else(|| panic!("ICE: Expected hoisted constant allocations to be set for function {brillig_function_id}"));
            hoisted_constants_allocations.extend(hoisted_allocations);
        } else {
            unreachable!(
                "ICE: Expected global allocation to be set for function {brillig_function_id}"
            );
        }
        (globals_allocations, hoisted_constants_allocations)
    }
}

pub(crate) type BrilligGlobalsArtifact = (
    BrilligArtifact<FieldElement>,
    HashMap<ValueId, BrilligVariable>,
    usize,
    HashMap<(FieldElement, NumericType), BrilligVariable>,
);

pub(crate) fn convert_ssa_globals(
    enable_debug_trace: bool,
    globals_dfg: &DataFlowGraph,
    used_globals: &HashSet<ValueId>,
    hoisted_global_constants: &BTreeSet<(FieldElement, NumericType)>,
    entry_point: FunctionId,
) -> BrilligGlobalsArtifact {
    let mut brillig_context = BrilligContext::new_for_global_init(enable_debug_trace, entry_point);
    // The global space does not have globals itself
    let empty_globals = HashMap::default();
    // We can use any ID here as this context is only going to be used for globals which does not differentiate
    // by functions and blocks. The only Label that should be used in the globals context is `Label::globals_init()`
    let mut function_context = FunctionContext::default();
    brillig_context.enter_context(Label::globals_init(entry_point));

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

    let hoisted_global_constants =
        brillig_block.compile_globals(globals_dfg, used_globals, hoisted_global_constants);

    let globals_size = brillig_context.global_space_size();

    brillig_context.return_instruction();

    let artifact = brillig_context.artifact();
    (artifact, function_context.ssa_value_allocations, globals_size, hoisted_global_constants)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use acvm::{
        acir::brillig::{BitSize, IntegerBitSize, Opcode},
        FieldElement,
    };

    use crate::brillig::{brillig_ir::registers::RegisterAllocator, GlobalSpace, LabelType, Ssa};

    use super::ConstantAllocation;

    #[test]
    fn entry_points_different_globals() {
        let src = "
        g0 = Field 2

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
            v2 = add v1, Field 2
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        // Need to run DIE to generate the used globals map, which is necessary for Brillig globals generation.
        let mut ssa = ssa.dead_instruction_elimination();

        let used_globals_map = std::mem::take(&mut ssa.used_globals);
        let brillig = ssa.to_brillig_with_globals(false, used_globals_map);

        assert_eq!(
            brillig.globals.len(),
            2,
            "Should have a globals artifact associated with each entry point"
        );
        for (func_id, mut artifact) in brillig.globals {
            let labels = artifact.take_labels();
            // When entering a context two labels are created.
            // One is a context label and another is a section label.
            assert_eq!(labels.len(), 2);
            for (label, position) in labels {
                assert_eq!(label.label_type, LabelType::GlobalInit(func_id));
                assert_eq!(position, 0);
            }
            if func_id.to_u32() == 1 {
                assert_eq!(
                    artifact.byte_code.len(),
                    1,
                    "Expected just a `Return`, but got more than a single opcode"
                );
                assert!(matches!(&artifact.byte_code[0], Opcode::Return));
            } else if func_id.to_u32() == 2 {
                assert_eq!(
                    artifact.byte_code.len(),
                    2,
                    "Expected enough opcodes to initialize the globals"
                );
                let Opcode::Const { destination, bit_size, value } = &artifact.byte_code[0] else {
                    panic!("First opcode is expected to be `Const`");
                };
                assert_eq!(destination.unwrap_direct(), GlobalSpace::start());
                assert!(matches!(bit_size, BitSize::Field));
                assert_eq!(*value, FieldElement::from(2u128));

                assert!(matches!(&artifact.byte_code[1], Opcode::Return));
            } else {
                panic!("Unexpected function id: {func_id}");
            }
        }
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
        // Need to run DIE to generate the used globals map, which is necessary for Brillig globals generation.
        let mut ssa = ssa.dead_instruction_elimination();

        let used_globals_map = std::mem::take(&mut ssa.used_globals);
        let brillig = ssa.to_brillig_with_globals(false, used_globals_map);

        assert_eq!(
            brillig.globals.len(),
            3,
            "Should have a globals artifact associated with each entry point"
        );
        for (func_id, mut artifact) in brillig.globals {
            let labels = artifact.take_labels();
            // When entering a context two labels are created.
            // One is a context label and another is a section label.
            assert_eq!(labels.len(), 2);
            for (label, position) in labels {
                assert_eq!(label.label_type, LabelType::GlobalInit(func_id));
                assert_eq!(position, 0);
            }
            if func_id.to_u32() == 1 {
                assert_eq!(
                    artifact.byte_code.len(),
                    2,
                    "Expected enough opcodes to initialize the globals"
                );
                let Opcode::Const { destination, bit_size, value } = &artifact.byte_code[0] else {
                    panic!("First opcode is expected to be `Const`");
                };
                assert_eq!(destination.unwrap_direct(), GlobalSpace::start());
                assert!(matches!(bit_size, BitSize::Field));
                assert_eq!(*value, FieldElement::from(1u128));

                assert!(matches!(&artifact.byte_code[1], Opcode::Return));
            } else if func_id.to_u32() == 2 || func_id.to_u32() == 3 {
                // We want the entry point which uses globals (f2) and the entry point which calls f2 function internally (f3 through f4)
                // to have the same globals initialized.
                assert_eq!(
                    artifact.byte_code.len(),
                    30,
                    "Expected enough opcodes to initialize the globals"
                );
                let globals_max_memory = brillig
                    .globals_memory_size
                    .get(&func_id)
                    .copied()
                    .expect("Should have globals memory size");
                assert_eq!(globals_max_memory, 7);
            } else {
                panic!("Unexpected function id: {func_id}");
            }
        }
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
        // Need to run DIE to generate the used globals map, which is necessary for Brillig globals generation.
        let mut ssa = ssa.dead_instruction_elimination();

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

        let used_globals_map = std::mem::take(&mut ssa.used_globals);
        let brillig = ssa.to_brillig_with_globals(false, used_globals_map);

        assert_eq!(brillig.globals.len(), 1, "Should have a single entry point");
        for (func_id, artifact) in brillig.globals {
            assert_eq!(func_id.to_u32(), 1);
            assert_eq!(
                artifact.byte_code.len(),
                3,
                "Expected enough opcodes to initialize the hoisted constants"
            );
            let Opcode::Const { destination, bit_size, value } = &artifact.byte_code[0] else {
                panic!("First opcode is expected to be `Const`");
            };
            assert_eq!(destination.unwrap_direct(), GlobalSpace::start());
            assert!(matches!(bit_size, BitSize::Integer(IntegerBitSize::U1)));
            assert_eq!(*value, FieldElement::from(0u128));

            let Opcode::Const { destination, bit_size, value } = &artifact.byte_code[1] else {
                panic!("First opcode is expected to be `Const`");
            };
            assert_eq!(destination.unwrap_direct(), GlobalSpace::start() + 1);
            assert!(matches!(bit_size, BitSize::Field));
            assert_eq!(*value, FieldElement::from(1u128));

            assert!(matches!(&artifact.byte_code[2], Opcode::Return));
        }
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
        // Need to run DIE to generate the used globals map, which is necessary for Brillig globals generation.
        let mut ssa = ssa.dead_instruction_elimination();

        let used_globals_map = std::mem::take(&mut ssa.used_globals);
        let brillig = ssa.to_brillig_with_globals(false, used_globals_map);

        assert_eq!(
            brillig.globals.len(),
            2,
            "Should have a globals artifact associated with each entry point"
        );
        for (func_id, mut artifact) in brillig.globals {
            let labels = artifact.take_labels();
            // When entering a context two labels are created.
            // One is a context label and another is a section label.
            assert_eq!(labels.len(), 2);
            for (label, position) in labels {
                assert_eq!(label.label_type, LabelType::GlobalInit(func_id));
                assert_eq!(position, 0);
            }
            assert_eq!(
                artifact.byte_code.len(),
                1,
                "Expected enough opcodes to initialize the hoisted constants"
            );
            assert!(matches!(&artifact.byte_code[0], Opcode::Return));
        }
    }
}
