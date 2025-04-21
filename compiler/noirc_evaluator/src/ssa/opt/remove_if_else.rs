use std::collections::hash_map::Entry;

use acvm::{FieldElement, acir::AcirField};
use fxhash::FxHashMap as HashMap;

use crate::ssa::ir::function::RuntimeType;
use crate::ssa::ir::instruction::Hint;
use crate::ssa::ir::types::NumericType;
use crate::ssa::ir::value::ValueId;
use crate::ssa::{
    Ssa,
    ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, Intrinsic},
        types::Type,
        value::Value,
    },
    opt::flatten_cfg::value_merger::ValueMerger,
};

impl Ssa {
    /// This pass removes `inc_rc` and `dec_rc` instructions
    /// as long as there are no `array_set` instructions to an array
    /// of the same type in between.
    ///
    /// Note that this pass is very conservative since the array_set
    /// instruction does not need to be to the same array. This is because
    /// the given array may alias another array (e.g. function parameters or
    /// a `load`ed array from a reference).
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_if_else(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_if_else();
        }
        self
    }
}

impl Function {
    pub(crate) fn remove_if_else(&mut self) {
        // This should match the check in flatten_cfg
        if matches!(self.runtime(), RuntimeType::Brillig(_)) {
            // skip
        } else {
            Context::default().remove_if_else(self);
        }
    }
}

#[derive(Default)]
struct Context {
    slice_sizes: HashMap<ValueId, u32>,

    // Maps array_set result -> enable_side_effects_if value which was active during it.
    array_set_conditionals: HashMap<ValueId, ValueId>,
}

impl Context {
    fn remove_if_else(&mut self, function: &mut Function) {
        let block = function.entry_block();

        // Make sure this optimization runs when there's only one block
        assert_eq!(function.dfg[block].successors().count(), 0);

        let one = FieldElement::one();
        let mut current_conditional = function.dfg.make_constant(one, NumericType::bool());

        function.simple_reachable_blocks_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            match instruction {
                Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                    let then_condition = *then_condition;
                    let else_condition = *else_condition;
                    let then_value = *then_value;
                    let else_value = *else_value;

                    let typ = context.dfg.type_of_value(then_value);
                    assert!(!matches!(typ, Type::Numeric(_)));

                    let call_stack = context.dfg.get_instruction_call_stack_id(instruction_id);
                    let mut value_merger = ValueMerger::new(
                        context.dfg,
                        block,
                        &mut self.slice_sizes,
                        &mut self.array_set_conditionals,
                        Some(current_conditional),
                        call_stack,
                    );

                    let value = value_merger.merge_values(
                        then_condition,
                        else_condition,
                        then_value,
                        else_value,
                    );

                    let _typ = context.dfg.type_of_value(value);
                    let results = context.dfg.instruction_results(instruction_id);
                    let result = results[0];
                    // let result = match typ {
                    //     Type::Array(..) => results[0],
                    //     Type::Slice(..) => results[1],
                    //     other => unreachable!("IfElse instructions should only have arrays or slices at this point. Found {other:?}"),
                    // };

                    context.remove_current_instruction();
                    context.replace_value(result, value);
                    self.array_set_conditionals.insert(value, current_conditional);
                }
                Instruction::Call { func, arguments } => {
                    if let Value::Intrinsic(intrinsic) = context.dfg[*func] {
                        let results = context.dfg.instruction_results(instruction_id);

                        match slice_capacity_change(context.dfg, intrinsic, arguments, results) {
                            SizeChange::None => (),
                            SizeChange::SetTo(value, new_capacity) => {
                                self.slice_sizes.insert(value, new_capacity);
                            }
                            SizeChange::Inc { old, new } => {
                                let old_capacity = self.get_or_find_capacity(context.dfg, old);
                                self.slice_sizes.insert(new, old_capacity + 1);
                            }
                            SizeChange::Dec { old, new } => {
                                let old_capacity = self.get_or_find_capacity(context.dfg, old);
                                // We use a saturating sub here as calling `pop_front` or `pop_back` on a zero-length slice
                                // would otherwise underflow.
                                self.slice_sizes.insert(new, old_capacity.saturating_sub(1));
                            }
                        }
                    }
                }
                Instruction::ArraySet { array, .. } => {
                    let results = context.dfg.instruction_results(instruction_id);
                    let result = if results.len() == 2 { results[1] } else { results[0] };

                    self.array_set_conditionals.insert(result, current_conditional);

                    let old_capacity = self.get_or_find_capacity(context.dfg, *array);
                    self.slice_sizes.insert(result, old_capacity);
                }
                Instruction::EnableSideEffectsIf { condition } => {
                    current_conditional = *condition;
                }
                _ => (),
            }
        });
    }

    fn get_or_find_capacity(&mut self, dfg: &DataFlowGraph, value: ValueId) -> u32 {
        match self.slice_sizes.entry(value) {
            Entry::Occupied(entry) => return *entry.get(),
            Entry::Vacant(entry) => {
                if let Some((array, typ)) = dfg.get_array_constant(value) {
                    let length = array.len() / typ.element_types().len();
                    return *entry.insert(length as u32);
                }

                if let Type::Array(_, length) = dfg.type_of_value(value) {
                    return *entry.insert(length);
                }
            }
        }

        let dbg_value = &dfg[value];
        unreachable!("No size for slice {value} = {dbg_value:?}")
    }
}

enum SizeChange {
    None,
    SetTo(ValueId, u32),

    // These two variants store the old and new slice ids
    // not their lengths which should be old_len = new_len +/- 1
    Inc { old: ValueId, new: ValueId },
    Dec { old: ValueId, new: ValueId },
}

/// Find the change to a slice's capacity an instruction would have
fn slice_capacity_change(
    dfg: &DataFlowGraph,
    intrinsic: Intrinsic,
    arguments: &[ValueId],
    results: &[ValueId],
) -> SizeChange {
    match intrinsic {
        Intrinsic::SlicePushBack | Intrinsic::SlicePushFront | Intrinsic::SliceInsert => {
            // Expecting:  len, slice = ...
            assert_eq!(results.len(), 2);
            let old = arguments[1];
            let new = results[1];
            assert!(matches!(dfg.type_of_value(old), Type::Slice(_)));
            assert!(matches!(dfg.type_of_value(new), Type::Slice(_)));
            SizeChange::Inc { old, new }
        }

        Intrinsic::SlicePopBack | Intrinsic::SliceRemove => {
            let old = arguments[1];
            let new = results[1];
            assert!(matches!(dfg.type_of_value(old), Type::Slice(_)));
            assert!(matches!(dfg.type_of_value(new), Type::Slice(_)));
            SizeChange::Dec { old, new }
        }

        Intrinsic::SlicePopFront => {
            let old = arguments[1];
            let new = results[results.len() - 1];
            assert!(matches!(dfg.type_of_value(old), Type::Slice(_)));
            assert!(matches!(dfg.type_of_value(new), Type::Slice(_)));
            SizeChange::Dec { old, new }
        }

        Intrinsic::AsSlice => {
            assert_eq!(arguments.len(), 1);
            assert_eq!(results.len(), 2);
            let length = match dfg.type_of_value(arguments[0]) {
                Type::Array(_, length) => length,
                other => unreachable!("slice_capacity_change expected array, found {other:?}"),
            };
            assert!(matches!(dfg.type_of_value(results[1]), Type::Slice(_)));
            SizeChange::SetTo(results[1], length)
        }

        // These cases don't affect slice capacities
        Intrinsic::AssertConstant
        | Intrinsic::StaticAssert
        | Intrinsic::ApplyRangeConstraint
        | Intrinsic::ArrayLen
        | Intrinsic::ArrayAsStrUnchecked
        | Intrinsic::StrAsBytes
        | Intrinsic::BlackBox(_)
        | Intrinsic::Hint(Hint::BlackBox)
        | Intrinsic::AsWitness
        | Intrinsic::IsUnconstrained
        | Intrinsic::DerivePedersenGenerators
        | Intrinsic::ToBits(_)
        | Intrinsic::ToRadix(_)
        | Intrinsic::ArrayRefCount
        | Intrinsic::SliceRefCount
        | Intrinsic::FieldLessThan => SizeChange::None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn merge_basic_arrays() {
        // This is the flattened SSA for the following Noir logic:
        // ```
        // fn main(x: bool, mut y: [u32; 2]) {
        //     if x {
        //         y[0] = 1;
        //         y[1] = 2;
        //     }
        //
        //     let z = y[0] + y[1];
        //     assert(z == 3);
        // }
        // ```
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 1
            v7 = array_set v5, index u32 1, value u32 2
            v8 = not v0
            v9 = if v0 then v7 else (if v8) v1
            enable_side_effects u1 1
            v11 = array_get v9, index u32 0 -> u32
            v12 = array_get v9, index u32 1 -> u32
            v13 = add v11, v12
            v15 = eq v13, u32 3
            constrain v13 == u32 3
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else();

        // In case our if block is never activated, we need to fetch each value from the original array.
        // We then should create a new array where each value can be mapped to `(then_condition * then_value) + (!then_condition * else_value)`.
        // The `then_value` and `else_value` for an array will be every element of the array. Thus, we should see array_get operations
        // on the original array as well as the new values we are writing to the array.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 1
            v7 = array_set v5, index u32 1, value u32 2
            v8 = not v0
            v10 = array_get v1, index Field 0 -> u32
            v11 = cast v0 as u32
            v12 = cast v8 as u32
            v13 = unchecked_mul v12, v10
            v14 = unchecked_add v11, v13
            v16 = array_get v1, index Field 1 -> u32
            v17 = cast v0 as u32
            v18 = cast v8 as u32
            v19 = unchecked_mul v17, u32 2
            v20 = unchecked_mul v18, v16
            v21 = unchecked_add v19, v20
            v22 = make_array [v14, v21] : [u32; 2]
            enable_side_effects u1 1
            v24 = array_get v22, index u32 0 -> u32
            v25 = array_get v22, index u32 1 -> u32
            v26 = add v24, v25
            v28 = eq v26, u32 3
            constrain v26 == u32 3
            return
        }
        "#);
    }

    #[test]
    fn try_merge_only_changed_indices() {
        // This is the flattened SSA for the following Noir logic:
        // ```
        // fn main(x: bool, mut y: [u32; 2]) {
        //     if x {
        //         y[0] = 1;
        //     }
        //
        //     let z = y[0] + y[1];
        //     assert(z == 1);
        // }
        // ```
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 1
            v6 = not v0
            v7 = if v0 then v5 else (if v6) v1
            enable_side_effects u1 1
            v9 = array_get v7, index u32 0 -> u32
            v10 = array_get v7, index u32 1 -> u32
            v11 = add v9, v10
            v12 = eq v11, u32 1
            constrain v11 == u32 1
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.remove_if_else();

        // We attempt to optimize array mergers to only handle where an array was modified,
        // rather than merging the entire array. As we only modify the `y` array at a single index,
        // we instead only map the if predicate onto the the numeric value we are looking to write,
        // and then write into the array directly.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: [u32; 2]):
            v2 = allocate -> &mut [u32; 2]
            enable_side_effects v0
            v5 = array_set v1, index u32 0, value u32 1
            v6 = not v0
            enable_side_effects v0
            v7 = array_get v1, index u32 0 -> u32
            v8 = cast v0 as u32
            v9 = cast v6 as u32
            v10 = unchecked_mul v9, v7
            v11 = unchecked_add v8, v10
            v12 = array_set v5, index u32 0, value v11
            enable_side_effects v0
            enable_side_effects u1 1
            v14 = array_get v12, index u32 0 -> u32
            v15 = array_get v12, index u32 1 -> u32
            v16 = add v14, v15
            v17 = eq v16, u32 1
            constrain v16 == u32 1
            return
        }
        "#);
    }

    #[test]
    fn try_merge_only_changed_indices_2() {
        let src = "
        g0 = u32 2

        acir(inline) predicate_pure fn main f0 {
          b0(v1: [Field; 2], v2: [Field; 2]):
            v4 = array_get v1, index u32 0 -> Field
            v6 = array_get v1, index u32 1 -> Field
            v7 = array_get v2, index u32 0 -> Field
            v8 = array_get v2, index u32 1 -> Field
            v9 = make_array [v4, v6, v7, v8] : [Field; 4]
            v11 = make_array [Field 0, Field 0] : [Field; 2]
            v12 = make_array [Field 0, Field 0] : [Field; 2]
            v13 = allocate -> &mut [Field; 2]
            v14 = allocate -> &mut u32
            v15 = allocate -> &mut [Field; 2]
            v16 = allocate -> &mut u32
            v17 = array_get v1, index u32 0 -> Field
            v18 = eq v17, Field 0
            v19 = not v18
            enable_side_effects v19
            v20 = array_get v1, index u32 0 -> Field
            v21 = make_array [v20, Field 0] : [Field; 2]
            v22 = if v19 then v21 else (if v18) v11
            v23 = cast v19 as u32
            v24 = cast v18 as u32
            enable_side_effects u1 1
            v26 = array_get v1, index u32 1 -> Field
            v27 = eq v26, Field 0
            v28 = not v27
            enable_side_effects v28
            v29 = array_get v1, index u32 1 -> Field
            v30 = lt v23, u32 2
            v31 = mul v30, v28
            constrain v31 == v28
            v32 = lt v23, u32 2
            v33 = mul v32, v28
            constrain v33 == v28
            v34 = array_set v22, index v23, value v29
            v35 = add v23, u32 1
            v36 = if v28 then v34 else (if v27) v22
            v37 = cast v28 as u32
            v38 = cast v27 as u32
            v39 = unchecked_mul v37, v35
            v40 = unchecked_mul v38, v23
            v41 = unchecked_add v39, v40
            enable_side_effects u1 1
            v42 = array_get v2, index u32 0 -> Field
            v43 = eq v42, Field 0
            v44 = not v43
            enable_side_effects v44
            v45 = array_get v2, index u32 0 -> Field
            v46 = make_array [v45, Field 0] : [Field; 2]
            v47 = if v44 then v46 else (if v43) v12
            v48 = cast v44 as u32
            v49 = cast v43 as u32
            enable_side_effects u1 1
            v50 = array_get v2, index u32 1 -> Field
            v51 = eq v50, Field 0
            v52 = not v51
            enable_side_effects v52
            v53 = array_get v2, index u32 1 -> Field
            v54 = lt v48, u32 2
            v55 = mul v54, v52
            constrain v55 == v52
            v56 = lt v48, u32 2
            v57 = mul v56, v52
            constrain v57 == v52
            v58 = array_set v47, index v48, value v53
            v59 = add v48, u32 1
            v60 = if v52 then v58 else (if v51) v47
            v61 = cast v52 as u32
            v62 = cast v51 as u32
            v63 = unchecked_mul v61, v59
            v64 = unchecked_mul v62, v48
            v65 = unchecked_add v63, v64
            enable_side_effects u1 1
            v66 = array_get v36, index u32 0 -> Field
            v67 = array_get v36, index u32 1 -> Field
            v68 = array_get v60, index u32 0 -> Field
            v69 = array_get v60, index u32 1 -> Field
            v70 = make_array [v66, v67, v68, v69] : [Field; 4]
            return v70
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_if_else();
        assert_ssa_snapshot!(ssa, @r"
        g0 = u32 2

        acir(inline) predicate_pure fn main f0 {
          b0(v1: [Field; 2], v2: [Field; 2]):
            v4 = array_get v1, index u32 0 -> Field
            v6 = array_get v1, index u32 1 -> Field
            v7 = array_get v2, index u32 0 -> Field
            v8 = array_get v2, index u32 1 -> Field
            v9 = make_array [v4, v6, v7, v8] : [Field; 4]
            v11 = make_array [Field 0, Field 0] : [Field; 2]
            v12 = make_array [Field 0, Field 0] : [Field; 2]
            v13 = allocate -> &mut [Field; 2]
            v14 = allocate -> &mut u32
            v15 = allocate -> &mut [Field; 2]
            v16 = allocate -> &mut u32
            v17 = array_get v1, index u32 0 -> Field
            v18 = eq v17, Field 0
            v19 = not v18
            enable_side_effects v19
            v20 = array_get v1, index u32 0 -> Field
            v21 = make_array [v20, Field 0] : [Field; 2]
            v22 = cast v19 as Field
            v23 = cast v18 as Field
            v24 = mul v22, v20
            v25 = make_array [v24, Field 0] : [Field; 2]
            v26 = cast v19 as u32
            v27 = cast v18 as u32
            enable_side_effects u1 1
            v29 = array_get v1, index u32 1 -> Field
            v30 = eq v29, Field 0
            v31 = not v30
            enable_side_effects v31
            v32 = array_get v1, index u32 1 -> Field
            v33 = lt v26, u32 2
            v34 = mul v33, v31
            constrain v34 == v31
            v35 = lt v26, u32 2
            v36 = mul v35, v31
            constrain v36 == v31
            v37 = array_set v25, index v26, value v32
            v38 = add v26, u32 1
            enable_side_effects v31
            v39 = array_get v37, index v26 -> Field
            v40 = array_get v25, index v26 -> Field
            v41 = cast v31 as Field
            v42 = cast v30 as Field
            v43 = mul v41, v39
            v44 = mul v42, v40
            v45 = add v43, v44
            v46 = array_set v37, index v26, value v45
            enable_side_effects v31
            v47 = cast v31 as u32
            v48 = cast v30 as u32
            v49 = unchecked_mul v47, v38
            v50 = unchecked_mul v48, v26
            v51 = unchecked_add v49, v50
            enable_side_effects u1 1
            v52 = array_get v2, index u32 0 -> Field
            v53 = eq v52, Field 0
            v54 = not v53
            enable_side_effects v54
            v55 = array_get v2, index u32 0 -> Field
            v56 = make_array [v55, Field 0] : [Field; 2]
            v57 = cast v54 as Field
            v58 = cast v53 as Field
            v59 = mul v57, v55
            v60 = make_array [v59, Field 0] : [Field; 2]
            v61 = cast v54 as u32
            v62 = cast v53 as u32
            enable_side_effects u1 1
            v63 = array_get v2, index u32 1 -> Field
            v64 = eq v63, Field 0
            v65 = not v64
            enable_side_effects v65
            v66 = array_get v2, index u32 1 -> Field
            v67 = lt v61, u32 2
            v68 = mul v67, v65
            constrain v68 == v65
            v69 = lt v61, u32 2
            v70 = mul v69, v65
            constrain v70 == v65
            v71 = array_set v60, index v61, value v66
            v72 = add v61, u32 1
            enable_side_effects v65
            v73 = array_get v71, index v61 -> Field
            v74 = array_get v60, index v61 -> Field
            v75 = cast v65 as Field
            v76 = cast v64 as Field
            v77 = mul v75, v73
            v78 = mul v76, v74
            v79 = add v77, v78
            v80 = array_set v71, index v61, value v79
            enable_side_effects v65
            v81 = cast v65 as u32
            v82 = cast v64 as u32
            v83 = unchecked_mul v81, v72
            v84 = unchecked_mul v82, v61
            v85 = unchecked_add v83, v84
            enable_side_effects u1 1
            v86 = array_get v46, index u32 0 -> Field
            v87 = array_get v46, index u32 1 -> Field
            v88 = array_get v80, index u32 0 -> Field
            v89 = array_get v80, index u32 1 -> Field
            v90 = make_array [v86, v87, v88, v89] : [Field; 4]
            return v90
        }
        ");
    }
}
