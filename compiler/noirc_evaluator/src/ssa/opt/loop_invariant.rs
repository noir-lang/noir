//! The loop invariant code motion pass moves code from inside a loop to before the loop
//! if that code will always have the same result on every iteration of the loop.
//!
//! To identify a loop invariant, check whether all of an instruction's values are:
//! - Outside of the loop
//! - Constant
//! - Already marked as loop invariants
//!
//! We also check that we are not hoisting instructions with side effects.
use fxhash::FxHashSet as HashSet;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::{Function, RuntimeType},
        function_inserter::FunctionInserter,
        instruction::InstructionId,
        value::ValueId,
    },
    Ssa,
};

use super::unrolling::{Loop, Loops};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn loop_invariant_code_motion(mut self) -> Ssa {
        let brillig_functions = self
            .functions
            .iter_mut()
            .filter(|(_, func)| matches!(func.runtime(), RuntimeType::Brillig(_)));

        for (_, function) in brillig_functions {
            function.loop_invariant_code_motion();
        }

        self
    }
}

impl Function {
    fn loop_invariant_code_motion(&mut self) {
        Loops::find_all(self).hoist_loop_invariants(self);
    }
}

impl Loops {
    fn hoist_loop_invariants(self, function: &mut Function) {
        let mut context = LoopInvariantContext::new(function);

        for loop_ in self.yet_to_unroll.iter() {
            let Ok(pre_header) = loop_.get_pre_header(context.inserter.function, &self.cfg) else {
                // If the loop does not have a preheader we skip hoisting loop invariants for this loop
                continue;
            };
            context.hoist_loop_invariants(loop_, pre_header);
        }

        context.map_dependent_instructions();
    }
}

struct LoopInvariantContext<'f> {
    inserter: FunctionInserter<'f>,
    defined_in_loop: HashSet<ValueId>,
    loop_invariants: HashSet<ValueId>,
}

impl<'f> LoopInvariantContext<'f> {
    fn new(function: &'f mut Function) -> Self {
        Self {
            inserter: FunctionInserter::new(function),
            defined_in_loop: HashSet::default(),
            loop_invariants: HashSet::default(),
        }
    }

    fn hoist_loop_invariants(&mut self, loop_: &Loop, pre_header: BasicBlockId) {
        self.set_values_defined_in_loop(loop_);

        for block in loop_.blocks.iter() {
            for instruction_id in self.inserter.function.dfg[*block].take_instructions() {
                let hoist_invariant = self.can_hoist_invariant(instruction_id);

                if hoist_invariant {
                    self.inserter.push_instruction(instruction_id, pre_header);
                } else {
                    self.inserter.push_instruction(instruction_id, *block);
                }

                self.update_values_defined_in_loop_and_invariants(instruction_id, hoist_invariant);
            }
        }
    }

    /// Gather the variables declared within the loop
    fn set_values_defined_in_loop(&mut self, loop_: &Loop) {
        for block in loop_.blocks.iter() {
            let params = self.inserter.function.dfg.block_parameters(*block);
            self.defined_in_loop.extend(params);
            for instruction_id in self.inserter.function.dfg[*block].instructions() {
                let results = self.inserter.function.dfg.instruction_results(*instruction_id);
                self.defined_in_loop.extend(results);
            }
        }
    }

    /// Update any values defined in the loop and loop invariants after a
    /// analyzing and re-inserting a loop's instruction.
    fn update_values_defined_in_loop_and_invariants(
        &mut self,
        instruction_id: InstructionId,
        hoist_invariant: bool,
    ) {
        let results = self.inserter.function.dfg.instruction_results(instruction_id).to_vec();
        // We will have new IDs after pushing instructions.
        // We should mark the resolved result IDs as also being defined within the loop.
        let results =
            results.into_iter().map(|value| self.inserter.resolve(value)).collect::<Vec<_>>();
        self.defined_in_loop.extend(results.iter());

        // We also want the update result IDs when we are marking loop invariants as we may not
        // be going through the blocks of the loop in execution order
        if hoist_invariant {
            // Track already found loop invariants
            self.loop_invariants.extend(results.iter());
        }
    }

    fn can_hoist_invariant(&mut self, instruction_id: InstructionId) -> bool {
        let mut is_loop_invariant = true;
        // The list of blocks for a nested loop contain any inner loops as well.
        // We may have already re-inserted new instructions if two loops share blocks
        // so we need to map all the values in the instruction which we want to check.
        let (instruction, _) = self.inserter.map_instruction(instruction_id);
        instruction.for_each_value(|value| {
            // If an instruction value is defined in the loop and not already a loop invariant
            // the instruction results are not loop invariants.
            //
            // We are implicitly checking whether the values are constant as well.
            // The set of values defined in the loop only contains instruction results and block parameters
            // which cannot be constants.
            is_loop_invariant &=
                !self.defined_in_loop.contains(&value) || self.loop_invariants.contains(&value);
        });
        is_loop_invariant && instruction.can_be_deduplicated(&self.inserter.function.dfg, false)
    }

    fn map_dependent_instructions(&mut self) {
        let blocks = self.inserter.function.reachable_blocks();
        for block in blocks {
            for instruction_id in self.inserter.function.dfg[block].take_instructions() {
                self.inserter.push_instruction(instruction_id, block);
            }
            self.inserter.map_terminator_in_place(block);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ssa::opt::assert_normalized_ssa_equals;
    use crate::ssa::Ssa;

    #[test]
    fn simple_loop_invariant_code_motion() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
              jmp b1(u32 0)
          b1(v2: u32):
              v5 = lt v2, u32 4
              jmpif v5 then: b3, else: b2
          b3():
              v6 = mul v0, v1
              constrain v6 == u32 6
              v8 = add v2, u32 1
              jmp b1(v8)
          b2():
              return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        // `v6 = mul v0, v1` in b3 should now be `v3 = mul v0, v1` in b0
        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = mul v0, v1
            jmp b1(u32 0)
          b1(v2: u32):
            v6 = lt v2, u32 4
            jmpif v6 then: b3, else: b2
          b3():
            constrain v3 == u32 6
            v9 = add v2, u32 1
            jmp b1(v9)
          b2():
            return
        }
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn nested_loop_invariant_code_motion() {
        // Check that a loop invariant in the inner loop of a nested loop
        // is hoisted to the parent loop's pre-header block.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            jmp b1(u32 0)
          b1(v2: u32):
            v6 = lt v2, u32 4
            jmpif v6 then: b3, else: b2
          b3():
            jmp b4(u32 0)
          b4(v3: u32):
            v7 = lt v3, u32 4
            jmpif v7 then: b6, else: b5
          b6():
            v10 = mul v0, v1
            constrain v10 == u32 6
            v12 = add v3, u32 1
            jmp b4(v12)
          b5():
            v9 = add v2, u32 1
            jmp b1(v9)
          b2():
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        // `v10 = mul v0, v1` in b6 should now be `v4 = mul v0, v1` in b0
        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v4 = mul v0, v1
            jmp b1(u32 0)
          b1(v2: u32):
            v7 = lt v2, u32 4
            jmpif v7 then: b3, else: b2
          b3():
            jmp b4(u32 0)
          b4(v3: u32):
            v8 = lt v3, u32 4
            jmpif v8 then: b6, else: b5
          b6():
            constrain v4 == u32 6
            v12 = add v3, u32 1
            jmp b4(v12)
          b5():
            v10 = add v2, u32 1
            jmp b1(v10)
          b2():
            return
        }
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn hoist_invariant_with_invariant_as_argument() {
        // Check that an instruction which has arguments defined in the loop
        // but which are already marked loop invariants is still hoisted to the preheader.
        //
        // For example, in b3 we have the following instructions:
        // ```text
        // v6 = mul v0, v1
        // v7 = mul v6, v0
        // ```
        // `v6` should be marked a loop invariants as `v0` and `v1` are both declared outside of the loop.
        // As we will be hoisting `v6 = mul v0, v1` to the loop preheader we know that we can also
        // hoist `v7 = mul v6, v0`.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            jmp b1(u32 0)
          b1(v2: u32):
            v5 = lt v2, u32 4
            jmpif v5 then: b3, else: b2
          b3():
            v6 = mul v0, v1
            v7 = mul v6, v0
            v8 = eq v7, u32 12
            constrain v7 == u32 12
            v9 = add v2, u32 1
            jmp b1(v9)
          b2():
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = mul v0, v1
            v4 = mul v3, v0
            v6 = eq v4, u32 12
            jmp b1(u32 0)
          b1(v2: u32):
            v9 = lt v2, u32 4
            jmpif v9 then: b3, else: b2
          b3():
            constrain v4 == u32 12
            v11 = add v2, u32 1
            jmp b1(v11)
          b2():
            return
        }
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn do_not_hoist_instructions_with_side_effects() {
        // In `v12 = load v5` in `b3`, `v5` is defined outside the loop.
        // However, as the instruction has side effects, we want to make sure
        // we do not hoist the instruction to the loop preheader.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v4 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 5]
            inc_rc v4
            v5 = allocate -> &mut [u32; 5]
            store v4 at v5
            jmp b1(u32 0)
          b1(v2: u32):
            v7 = lt v2, u32 4
            jmpif v7 then: b3, else: b2
          b3():
            v12 = load v5 -> [u32; 5]
            v13 = array_set v12, index v0, value v1
            store v13 at v5
            v15 = add v2, u32 1
            jmp b1(v15)
          b2():
            v8 = load v5 -> [u32; 5]
            v10 = array_get v8, index u32 2 -> u32
            constrain v10 == u32 3
            return
        }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 4); // The final return is not counted

        let ssa = ssa.loop_invariant_code_motion();
        // The code should be unchanged
        assert_normalized_ssa_equals(ssa, src);
    }
}
