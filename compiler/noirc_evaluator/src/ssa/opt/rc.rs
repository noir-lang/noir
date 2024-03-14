use std::collections::{HashMap, HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        types::Type,
        value::ValueId,
    },
    ssa_gen::Ssa,
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
    pub(crate) fn remove_paired_rc(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            remove_paired_rc(function);
        }
        self
    }
}

#[derive(Default)]
struct Context {
    // All inc_rc instructions encountered without a corresponding dec_rc.
    // These are only searched for in the first block of a function.
    //
    // The type of the array being operated on is recorded.
    // If an array_set to that array type is encountered, that is also recorded.
    inc_rcs: HashMap<Type, Vec<IncRc>>,
}

struct IncRc {
    id: InstructionId,
    array: ValueId,
    possibly_mutated: bool,
}

/// This function is very simplistic for now. It takes advantage of the fact that dec_rc
/// instructions are currently issued only at the end of a function for parameters and will
/// only check the first and last block for inc & dec rc instructions to be removed. The rest
/// of the function is still checked for array_set instructions.
///
/// This restriction lets this function largely ignore merging intermediate results from other
/// blocks and handling loops.
fn remove_paired_rc(function: &mut Function) {
    // `dec_rc` is only issued for parameters currently so we can speed things
    // up a bit by skipping any functions without them.
    if !contains_array_parameter(function) {
        return;
    }

    let mut context = Context::default();

    context.find_rcs_in_entry_block(function);
    context.scan_for_array_sets(function);
    let to_remove = context.find_rcs_to_remove(function);
    remove_instructions(to_remove, function);
}

fn contains_array_parameter(function: &mut Function) -> bool {
    let mut parameters = function.parameters().iter();
    parameters.any(|parameter| function.dfg.type_of_value(*parameter).contains_an_array())
}

impl Context {
    fn find_rcs_in_entry_block(&mut self, function: &Function) {
        let entry = function.entry_block();

        for instruction in function.dfg[entry].instructions() {
            if let Instruction::IncrementRc { value } = &function.dfg[*instruction] {
                let typ = function.dfg.type_of_value(*value);

                // We assume arrays aren't mutated until we find an array_set
                let inc_rc = IncRc { id: *instruction, array: *value, possibly_mutated: false };
                self.inc_rcs.entry(typ).or_default().push(inc_rc);
            }
        }
    }

    /// Find each array_set instruction in the function and mark any arrays used
    /// by the inc_rc instructions as possibly mutated if they're the same type.
    fn scan_for_array_sets(&mut self, function: &Function) {
        for block in function.reachable_blocks() {
            for instruction in function.dfg[block].instructions() {
                if let Instruction::ArraySet { array, .. } = function.dfg[*instruction] {
                    let typ = function.dfg.type_of_value(array);
                    if let Some(inc_rcs) = self.inc_rcs.get_mut(&typ) {
                        for inc_rc in inc_rcs {
                            inc_rc.possibly_mutated = true;
                        }
                    }
                }
            }
        }
    }

    /// Find each dec_rc instruction and if the most recent inc_rc instruction for the same value
    /// is not possibly mutated, then we can remove them both. Returns each such pair.
    fn find_rcs_to_remove(&mut self, function: &Function) -> HashSet<InstructionId> {
        let last_block = Self::find_last_block(function);
        let mut to_remove = HashSet::new();

        for instruction in function.dfg[last_block].instructions() {
            if let Instruction::DecrementRc { value } = &function.dfg[*instruction] {
                if let Some(inc_rc) = self.pop_rc_for(*value, function) {
                    if !inc_rc.possibly_mutated {
                        to_remove.insert(inc_rc.id);
                        to_remove.insert(*instruction);
                    }
                }
            }
        }

        to_remove
    }

    /// Finds the block of the function with the Return instruction
    fn find_last_block(function: &Function) -> BasicBlockId {
        for block in function.reachable_blocks() {
            if matches!(
                function.dfg[block].terminator(),
                Some(TerminatorInstruction::Return { .. })
            ) {
                return block;
            }
        }

        unreachable!("SSA Function {} has no reachable return instruction!", function.id())
    }

    /// Finds and pops the IncRc for the given array value if possible.
    fn pop_rc_for(&mut self, value: ValueId, function: &Function) -> Option<IncRc> {
        let typ = function.dfg.type_of_value(value);

        let rcs = self.inc_rcs.get_mut(&typ)?;
        let position = rcs.iter().position(|inc_rc| inc_rc.array == value)?;

        Some(rcs.remove(position))
    }
}

fn remove_instructions(to_remove: HashSet<InstructionId>, function: &mut Function) {
    if !to_remove.is_empty() {
        for block in function.reachable_blocks() {
            function.dfg[block]
                .instructions_mut()
                .retain(|instruction| !to_remove.contains(instruction))
        }
    }
}
