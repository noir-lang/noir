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
            v4 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v5 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v7 = array_get v1, index u32 0 -> Field
            v8 = eq v7, Field 0
            v9 = not v8
            enable_side_effects v9
            v10 = array_get v1, index u32 0 -> Field
            v11 = make_array [v10, Field 0, Field 0, Field 0] : [Field; 4]
            v12 = if v9 then v11 else (if v8) v4
            v13 = cast v9 as u32
            v14 = cast v8 as u32
            enable_side_effects u1 1
            v17 = array_get v1, index u32 1 -> Field
            v18 = eq v17, Field 0
            v19 = not v18
            enable_side_effects v19
            v20 = array_get v1, index u32 1 -> Field
            v22 = lt v13, u32 4
            v23 = mul v22, v19
            constrain v23 == v19
            v24 = lt v13, u32 4
            v25 = mul v24, v19
            constrain v25 == v19
            v26 = array_set v12, index v13, value v20
            v27 = add v13, u32 1
            v28 = if v19 then v26 else (if v18) v12
            v29 = cast v19 as u32
            v30 = cast v18 as u32
            v31 = unchecked_mul v29, v27
            v32 = unchecked_mul v30, v13
            v33 = unchecked_add v31, v32
            enable_side_effects u1 1
            v34 = array_get v2, index u32 0 -> Field
            v35 = eq v34, Field 0
            v36 = not v35
            enable_side_effects v36
            v37 = array_get v2, index u32 0 -> Field
            v38 = make_array [v37, Field 0, Field 0, Field 0] : [Field; 4]
            v39 = if v36 then v38 else (if v35) v5
            v40 = cast v36 as u32
            v41 = cast v35 as u32
            enable_side_effects u1 1
            v42 = array_get v2, index u32 1 -> Field
            v43 = eq v42, Field 0
            v44 = not v43
            enable_side_effects v44
            v45 = array_get v2, index u32 1 -> Field
            v46 = lt v40, u32 4
            v47 = mul v46, v44
            constrain v47 == v44
            v48 = lt v40, u32 4
            v49 = mul v48, v44
            constrain v49 == v44
            v50 = array_set v39, index v40, value v45
            v51 = add v40, u32 1
            v52 = if v44 then v50 else (if v43) v39
            v53 = cast v44 as u32
            v54 = cast v43 as u32
            v55 = unchecked_mul v53, v51
            v56 = unchecked_mul v54, v40
            v57 = unchecked_add v55, v56
            enable_side_effects u1 1
            v58 = array_get v28, index u32 0 -> Field
            v59 = array_get v28, index u32 1 -> Field
            v60 = array_get v28, index u32 2 -> Field
            v62 = array_get v28, index u32 3 -> Field
            v63 = array_get v52, index u32 0 -> Field
            v64 = array_get v52, index u32 1 -> Field
            v65 = array_get v52, index u32 2 -> Field
            v66 = array_get v52, index u32 3 -> Field
            v67 = make_array [v58, v59, v60, v62, v63, v64, v65, v66] : [Field; 8]
            return v67
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_if_else();
        assert_ssa_snapshot!(ssa, @r"
        g0 = u32 2

        acir(inline) predicate_pure fn main f0 {
          b0(v1: [Field; 2], v2: [Field; 2]):
            v4 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v5 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v7 = array_get v1, index u32 0 -> Field
            v8 = eq v7, Field 0
            v9 = not v8
            enable_side_effects v9
            v10 = array_get v1, index u32 0 -> Field
            v11 = make_array [v10, Field 0, Field 0, Field 0] : [Field; 4]
            v12 = cast v9 as Field
            v13 = cast v8 as Field
            v14 = mul v12, v10
            v15 = make_array [v14, Field 0, Field 0, Field 0] : [Field; 4]
            v16 = cast v9 as u32
            v17 = cast v8 as u32
            enable_side_effects u1 1
            v20 = array_get v1, index u32 1 -> Field
            v21 = eq v20, Field 0
            v22 = not v21
            enable_side_effects v22
            v23 = array_get v1, index u32 1 -> Field
            v25 = lt v16, u32 4
            v26 = mul v25, v22
            constrain v26 == v22
            v27 = lt v16, u32 4
            v28 = mul v27, v22
            constrain v28 == v22
            v29 = array_set v15, index v16, value v23
            v30 = add v16, u32 1
            enable_side_effects v22
            v31 = array_get v29, index v16 -> Field
            v32 = array_get v15, index v16 -> Field
            v33 = cast v22 as Field
            v34 = cast v21 as Field
            v35 = mul v33, v31
            v36 = mul v34, v32
            v37 = add v35, v36
            v38 = array_set v29, index v16, value v37
            enable_side_effects v22
            v39 = cast v22 as u32
            v40 = cast v21 as u32
            v41 = unchecked_mul v39, v30
            v42 = unchecked_mul v40, v16
            v43 = unchecked_add v41, v42
            enable_side_effects u1 1
            v44 = array_get v2, index u32 0 -> Field
            v45 = eq v44, Field 0
            v46 = not v45
            enable_side_effects v46
            v47 = array_get v2, index u32 0 -> Field
            v48 = make_array [v47, Field 0, Field 0, Field 0] : [Field; 4]
            v49 = cast v46 as Field
            v50 = cast v45 as Field
            v51 = mul v49, v47
            v52 = make_array [v51, Field 0, Field 0, Field 0] : [Field; 4]
            v53 = cast v46 as u32
            v54 = cast v45 as u32
            enable_side_effects u1 1
            v55 = array_get v2, index u32 1 -> Field
            v56 = eq v55, Field 0
            v57 = not v56
            enable_side_effects v57
            v58 = array_get v2, index u32 1 -> Field
            v59 = lt v53, u32 4
            v60 = mul v59, v57
            constrain v60 == v57
            v61 = lt v53, u32 4
            v62 = mul v61, v57
            constrain v62 == v57
            v63 = array_set v52, index v53, value v58
            v64 = add v53, u32 1
            enable_side_effects v57
            v65 = array_get v63, index v53 -> Field
            v66 = array_get v52, index v53 -> Field
            v67 = cast v57 as Field
            v68 = cast v56 as Field
            v69 = mul v67, v65
            v70 = mul v68, v66
            v71 = add v69, v70
            v72 = array_set v63, index v53, value v71
            enable_side_effects v57
            v73 = cast v57 as u32
            v74 = cast v56 as u32
            v75 = unchecked_mul v73, v64
            v76 = unchecked_mul v74, v53
            v77 = unchecked_add v75, v76
            enable_side_effects u1 1
            v78 = array_get v38, index u32 0 -> Field
            v79 = array_get v38, index u32 1 -> Field
            v80 = array_get v38, index u32 2 -> Field
            v82 = array_get v38, index u32 3 -> Field
            v83 = array_get v72, index u32 0 -> Field
            v84 = array_get v72, index u32 1 -> Field
            v85 = array_get v72, index u32 2 -> Field
            v86 = array_get v72, index u32 3 -> Field
            v87 = make_array [v78, v79, v80, v82, v83, v84, v85, v86] : [Field; 8]
            return v87
        }
        ");
    }
}
