use std::collections::hash_map::Entry;

use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;

use crate::ssa::ir::value::ValueId;
use crate::ssa::{
    ir::{
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, Intrinsic},
        types::Type,
        value::Value,
    },
    opt::flatten_cfg::value_merger::ValueMerger,
    Ssa,
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
            // This should match the check in flatten_cfg
            if let crate::ssa::ir::function::RuntimeType::Brillig = function.runtime() {
                continue;
            }

            Context::default().remove_if_else(function);
        }
        self
    }
}

#[derive(Default)]
struct Context {
    slice_sizes: HashMap<ValueId, usize>,

    // Maps array_set result -> element that was overwritten by that instruction.
    // Used to undo array_sets while merging values
    prev_array_set_elem_values: HashMap<ValueId, ValueId>,

    // Maps array_set result -> enable_side_effects_if value which was active during it.
    array_set_conditionals: HashMap<ValueId, ValueId>,
}

impl Context {
    fn remove_if_else(&mut self, function: &mut Function) {
        let block = function.entry_block();
        let instructions = function.dfg[block].take_instructions();
        let mut current_conditional = function.dfg.make_constant(FieldElement::one(), Type::bool());

        for instruction in instructions {
            match &function.dfg[instruction] {
                Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                    let then_condition = *then_condition;
                    let then_value = *then_value;
                    let else_condition = *else_condition;
                    let else_value = *else_value;

                    let typ = function.dfg.type_of_value(then_value);
                    assert!(!matches!(typ, Type::Numeric(_)));

                    let mut value_merger = ValueMerger::new(
                        &mut function.dfg,
                        block,
                        &mut self.slice_sizes,
                        &mut self.array_set_conditionals,
                        Some(current_conditional),
                    );

                    let value = value_merger.merge_values(
                        then_condition,
                        else_condition,
                        then_value,
                        else_value,
                    );

                    let _typ = function.dfg.type_of_value(value);
                    let results = function.dfg.instruction_results(instruction);
                    let result = results[0];
                    // let result = match typ {
                    //     Type::Array(..) => results[0],
                    //     Type::Slice(..) => results[1],
                    //     other => unreachable!("IfElse instructions should only have arrays or slices at this point. Found {other:?}"),
                    // };

                    function.dfg.set_value_from_id(result, value);
                    self.array_set_conditionals.insert(result, current_conditional);
                }
                Instruction::Call { func, arguments } => {
                    if let Value::Intrinsic(intrinsic) = function.dfg[*func] {
                        let results = function.dfg.instruction_results(instruction);

                        match slice_capacity_change(&function.dfg, intrinsic, arguments, results) {
                            SizeChange::None => (),
                            SizeChange::SetTo(value, new_capacity) => {
                                self.slice_sizes.insert(value, new_capacity);
                            }
                            SizeChange::Inc { old, new } => {
                                let old_capacity = self.get_or_find_capacity(&function.dfg, old);
                                self.slice_sizes.insert(new, old_capacity + 1);
                            }
                            SizeChange::Dec { old, new } => {
                                let old_capacity = self.get_or_find_capacity(&function.dfg, old);
                                self.slice_sizes.insert(new, old_capacity - 1);
                            }
                        }
                    }
                    function.dfg[block].instructions_mut().push(instruction);
                }
                Instruction::ArraySet { array, .. } => {
                    let results = function.dfg.instruction_results(instruction);
                    let result = if results.len() == 2 { results[1] } else { results[0] };

                    self.array_set_conditionals.insert(result, current_conditional);

                    let old_capacity = self.get_or_find_capacity(&function.dfg, *array);
                    self.slice_sizes.insert(result, old_capacity);
                    function.dfg[block].instructions_mut().push(instruction);
                }
                Instruction::EnableSideEffects { condition } => {
                    current_conditional = *condition;
                    function.dfg[block].instructions_mut().push(instruction);
                }
                _ => {
                    function.dfg[block].instructions_mut().push(instruction);
                }
            }
        }
    }

    fn get_or_find_capacity(&mut self, dfg: &DataFlowGraph, value: ValueId) -> usize {
        match self.slice_sizes.entry(value) {
            Entry::Occupied(entry) => return *entry.get(),
            Entry::Vacant(entry) => {
                if let Some((array, typ)) = dfg.get_array_constant(value) {
                    let length = array.len() / typ.element_types().len();
                    return *entry.insert(length);
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
    SetTo(ValueId, usize),

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

        Intrinsic::ToBits(_) => {
            assert_eq!(results.len(), 2);
            // Some tests fail this check, returning an array instead somehow:
            // assert!(matches!(dfg.type_of_value(results[1]), Type::Slice(_)));
            SizeChange::SetTo(results[1], FieldElement::max_num_bits() as usize)
        }
        // ToRadix seems to assume it is to bytes
        Intrinsic::ToRadix(_) => {
            assert_eq!(results.len(), 2);
            assert!(matches!(dfg.type_of_value(results[1]), Type::Slice(_)));
            SizeChange::SetTo(results[1], FieldElement::max_num_bytes() as usize)
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
        | Intrinsic::ApplyRangeConstraint
        | Intrinsic::ArrayLen
        | Intrinsic::StrAsBytes
        | Intrinsic::BlackBox(_)
        | Intrinsic::FromField
        | Intrinsic::AsField => SizeChange::None,
    }
}
