use std::collections::{BTreeMap, BTreeSet};

use acvm::FieldElement;
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
    BrilligArtifact, BrilligBlock, BrilligVariable, Function, FunctionContext, Label, ValueId,
};
use crate::{
    brillig::{brillig_ir::BrilligContext, Brillig, DataFlowGraph, FunctionId},
    ssa::opt::brillig_entry_points::{build_inner_call_to_entry_points, get_brillig_entry_points},
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
    brillig_entry_points: BTreeMap<FunctionId, BTreeSet<FunctionId>>,

    /// Maps an inner call to its Brillig entry point
    /// This is simply used to simplify fetching global allocations when compiling
    /// individual Brillig functions.
    inner_call_to_entry_point: HashMap<FunctionId, Vec<FunctionId>>,
    /// Final map that associated an entry point with its Brillig global allocations
    entry_point_globals_map: HashMap<FunctionId, SsaToBrilligGlobals>,
}

/// Mapping of SSA value ids to their Brillig allocations
pub(crate) type SsaToBrilligGlobals = HashMap<ValueId, BrilligVariable>;

impl BrilligGlobals {
    pub(crate) fn new(
        functions: &BTreeMap<FunctionId, Function>,
        mut used_globals: HashMap<FunctionId, HashSet<ValueId>>,
        main_id: FunctionId,
    ) -> Self {
        let brillig_entry_points = get_brillig_entry_points(functions, main_id);

        // Mark any globals used in a Brillig entry point.
        // Using the information collected we can determine which globals
        // an entry point must initialize.
        for (entry_point, entry_point_inner_calls) in brillig_entry_points.iter() {
            for inner_call in entry_point_inner_calls.iter() {
                let inner_globals = used_globals
                    .get(inner_call)
                    .expect("Should have a slot for each function")
                    .clone();
                used_globals
                    .get_mut(entry_point)
                    .expect("ICE: should have func")
                    .extend(inner_globals);
            }
        }

        let inner_call_to_entry_point = build_inner_call_to_entry_points(&brillig_entry_points);

        Self { used_globals, brillig_entry_points, inner_call_to_entry_point, ..Default::default() }
    }

    pub(crate) fn declare_globals(
        &mut self,
        globals_dfg: &DataFlowGraph,
        brillig: &mut Brillig,
        enable_debug_trace: bool,
    ) {
        let mut entry_point_globals_map = HashMap::default();
        // We only need to generate globals for entry points
        for (entry_point, _) in self.brillig_entry_points.iter() {
            let entry_point = *entry_point;

            let used_globals = self.used_globals.remove(&entry_point).unwrap_or_default();
            let (artifact, brillig_globals, globals_size) =
                convert_ssa_globals(enable_debug_trace, globals_dfg, &used_globals, entry_point);

            entry_point_globals_map.insert(entry_point, brillig_globals);

            brillig.globals.insert(entry_point, artifact);
            brillig.globals_memory_size.insert(entry_point, globals_size);
        }

        self.entry_point_globals_map = entry_point_globals_map;
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
    ) -> SsaToBrilligGlobals {
        let entry_points = self.inner_call_to_entry_point.get(&brillig_function_id);

        let mut globals_allocations = HashMap::default();
        if let Some(globals) = self.entry_point_globals_map.get(&brillig_function_id) {
            // Check whether `brillig_function_id` is itself an entry point.
            // If so, return the global allocations directly from `self.entry_point_globals_map`.
            globals_allocations.extend(globals);
            return globals_allocations;
        }

        if let Some(entry_points) = entry_points {
            assert!(self.entry_point_globals_map.get(&brillig_function_id).is_none());
            assert_eq!(entry_points.len(), 1, "{brillig_function_id} has multiple entry points");
            // A Brillig function is used by multiple entry points. Fetch both globals allocations
            // in case one is used by the internal call.
            let entry_point_allocations = entry_points
                .iter()
                .flat_map(|entry_point| self.entry_point_globals_map.get(entry_point))
                .collect::<Vec<_>>();
            for map in entry_point_allocations {
                globals_allocations.extend(map);
            }
        } else {
            unreachable!(
                "ICE: Expected global allocation to be set for function {brillig_function_id}"
            );
        }
        globals_allocations
    }
}

pub(crate) fn convert_ssa_globals(
    enable_debug_trace: bool,
    globals_dfg: &DataFlowGraph,
    used_globals: &HashSet<ValueId>,
    entry_point: FunctionId,
) -> (BrilligArtifact<FieldElement>, HashMap<ValueId, BrilligVariable>, usize) {
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
        building_globals: true,
    };

    brillig_block.compile_globals(globals_dfg, used_globals);

    let globals_size = brillig_context.global_space_size();

    brillig_context.return_instruction();

    let artifact = brillig_context.artifact();
    (artifact, function_context.ssa_value_allocations, globals_size)
}

#[cfg(test)]
mod tests {
    use acvm::{
        acir::brillig::{BitSize, Opcode},
        FieldElement,
    };

    use crate::brillig::{brillig_ir::registers::RegisterAllocator, GlobalSpace, LabelType, Ssa};

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
}
