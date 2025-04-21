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
            v11 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v12 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v17 = array_get v1, index u32 0 -> Field
            v18 = eq v17, Field 0
            v19 = not v18
            enable_side_effects v19
            v20 = array_get v1, index u32 0 -> Field
            v21 = make_array [v20, Field 0, Field 0, Field 0] : [Field; 4]
            v22 = if v19 then v21 else (if v18) v11
            v23 = cast v19 as u32
            v24 = cast v18 as u32
            enable_side_effects u1 1
            v26 = array_get v1, index u32 1 -> Field
            v27 = eq v26, Field 0
            v28 = not v27
            enable_side_effects v28
            v29 = array_get v1, index u32 1 -> Field
            v31 = lt v23, u32 4
            v32 = mul v31, v28
            constrain v32 == v28
            v33 = lt v23, u32 4
            v34 = mul v33, v28
            constrain v34 == v28
            v35 = array_set v22, index v23, value v29
            v36 = add v23, u32 1
            v37 = if v28 then v35 else (if v27) v22
            v38 = cast v28 as u32
            v39 = cast v27 as u32
            v40 = unchecked_mul v38, v36
            v41 = unchecked_mul v39, v23
            v42 = unchecked_add v40, v41
            enable_side_effects u1 1
            v43 = array_get v2, index u32 0 -> Field
            v44 = eq v43, Field 0
            v45 = not v44
            enable_side_effects v45
            v46 = array_get v2, index u32 0 -> Field
            v47 = make_array [v46, Field 0, Field 0, Field 0] : [Field; 4]
            v48 = if v45 then v47 else (if v44) v12
            v49 = cast v45 as u32
            v50 = cast v44 as u32
            enable_side_effects u1 1
            v51 = array_get v2, index u32 1 -> Field
            v52 = eq v51, Field 0
            v53 = not v52
            enable_side_effects v53
            v54 = array_get v2, index u32 1 -> Field
            v55 = lt v49, u32 4
            v56 = mul v55, v53
            constrain v56 == v53
            v57 = lt v49, u32 4
            v58 = mul v57, v53
            constrain v58 == v53
            v59 = array_set v48, index v49, value v54
            v60 = add v49, u32 1
            v61 = if v53 then v59 else (if v52) v48
            v62 = cast v53 as u32
            v63 = cast v52 as u32
            v64 = unchecked_mul v62, v60
            v65 = unchecked_mul v63, v49
            v66 = unchecked_add v64, v65
            enable_side_effects u1 1
            v67 = array_get v37, index u32 0 -> Field
            v68 = array_get v37, index u32 1 -> Field
            v69 = array_get v37, index u32 2 -> Field
            v71 = array_get v37, index u32 3 -> Field
            v72 = array_get v61, index u32 0 -> Field
            v73 = array_get v61, index u32 1 -> Field
            v74 = array_get v61, index u32 2 -> Field
            v75 = array_get v61, index u32 3 -> Field
            v76 = make_array [v67, v68, v69, v71, v72, v73, v74, v75] : [Field; 8]
            return v76
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
            v11 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v12 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v13 = array_get v1, index u32 0 -> Field
            v14 = eq v13, Field 0
            v15 = not v14
            enable_side_effects v15
            v16 = array_get v1, index u32 0 -> Field
            v17 = make_array [v16, Field 0, Field 0, Field 0] : [Field; 4]
            v18 = cast v15 as Field
            v19 = cast v14 as Field
            v20 = mul v18, v16
            v21 = make_array [v20, Field 0, Field 0, Field 0] : [Field; 4]
            v22 = cast v15 as u32
            v23 = cast v14 as u32
            enable_side_effects u1 1
            v25 = array_get v1, index u32 1 -> Field
            v26 = eq v25, Field 0
            v27 = not v26
            enable_side_effects v27
            v28 = array_get v1, index u32 1 -> Field
            v30 = lt v22, u32 4
            v31 = mul v30, v27
            constrain v31 == v27
            v32 = lt v22, u32 4
            v33 = mul v32, v27
            constrain v33 == v27
            v34 = array_set v21, index v22, value v28
            v35 = add v22, u32 1
            enable_side_effects v27
            v36 = array_get v34, index v22 -> Field
            v37 = array_get v21, index v22 -> Field
            v38 = cast v27 as Field
            v39 = cast v26 as Field
            v40 = mul v38, v36
            v41 = mul v39, v37
            v42 = add v40, v41
            v43 = array_set v34, index v22, value v42
            enable_side_effects v27
            v44 = cast v27 as u32
            v45 = cast v26 as u32
            v46 = unchecked_mul v44, v35
            v47 = unchecked_mul v45, v22
            v48 = unchecked_add v46, v47
            enable_side_effects u1 1
            v49 = array_get v2, index u32 0 -> Field
            v50 = eq v49, Field 0
            v51 = not v50
            enable_side_effects v51
            v52 = array_get v2, index u32 0 -> Field
            v53 = make_array [v52, Field 0, Field 0, Field 0] : [Field; 4]
            v54 = cast v51 as Field
            v55 = cast v50 as Field
            v56 = mul v54, v52
            v57 = make_array [v56, Field 0, Field 0, Field 0] : [Field; 4]
            v58 = cast v51 as u32
            v59 = cast v50 as u32
            enable_side_effects u1 1
            v60 = array_get v2, index u32 1 -> Field
            v61 = eq v60, Field 0
            v62 = not v61
            enable_side_effects v62
            v63 = array_get v2, index u32 1 -> Field
            v64 = lt v58, u32 4
            v65 = mul v64, v62
            constrain v65 == v62
            v66 = lt v58, u32 4
            v67 = mul v66, v62
            constrain v67 == v62
            v68 = array_set v57, index v58, value v63
            v69 = add v58, u32 1
            enable_side_effects v62
            v70 = array_get v68, index v58 -> Field
            v71 = array_get v57, index v58 -> Field
            v72 = cast v62 as Field
            v73 = cast v61 as Field
            v74 = mul v72, v70
            v75 = mul v73, v71
            v76 = add v74, v75
            v77 = array_set v68, index v58, value v76
            enable_side_effects v62
            v78 = cast v62 as u32
            v79 = cast v61 as u32
            v80 = unchecked_mul v78, v69
            v81 = unchecked_mul v79, v58
            v82 = unchecked_add v80, v81
            enable_side_effects u1 1
            v83 = array_get v43, index u32 0 -> Field
            v84 = array_get v43, index u32 1 -> Field
            v85 = array_get v43, index u32 2 -> Field
            v87 = array_get v43, index u32 3 -> Field
            v88 = array_get v77, index u32 0 -> Field
            v89 = array_get v77, index u32 1 -> Field
            v90 = array_get v77, index u32 2 -> Field
            v91 = array_get v77, index u32 3 -> Field
            v92 = make_array [v83, v84, v85, v87, v88, v89, v90, v91] : [Field; 8]
            return v92
        }
        ");
    }
}
