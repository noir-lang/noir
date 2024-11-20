//! The loop invariant code motion pass moves code from inside a loop to before the loop
//! if that code will always have the same result on every iteration of the loop.
//!
//! To identify a loop invariant, check whether all of an instruction's values are:
//! - Outside of the loop
//! - Constant
//! - Already marked as loop invariants
//!
//! We also check that we are not hoisting instructions with side effects.
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        function::{Function, RuntimeType},
        function_inserter::FunctionInserter,
        value::ValueId,
    },
    Ssa,
};

use super::{
    constant_folding::{push_instruction, replace_result_ids},
    unrolling::Loops,
};

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
        let mut inserter = FunctionInserter::new(function);

        for loop_ in self.yet_to_unroll {
            let Ok(pre_header) = loop_.get_pre_header(inserter.function, &self.cfg) else {
                // If the loop does not have a preheader we skip hoisting loop invariants for this loop
                continue;
            };

            // Gather the variables declared within the loop
            let mut defined_in_loop: HashSet<ValueId> = HashSet::default();
            for block in loop_.blocks.iter() {
                let params = inserter.function.dfg.block_parameters(*block);
                defined_in_loop.extend(params);
                for instruction_id in inserter.function.dfg[*block].instructions() {
                    let results = inserter.function.dfg.instruction_results(*instruction_id);
                    defined_in_loop.extend(results);
                }
            }

            // Instructions to move to the preheader
            let mut instructions_to_hoist = Vec::new();
            // Mapping to track unchanged instructions per block
            let mut block_to_instructions = HashMap::default();
            // Track already found loop invariants
            let mut loop_invariants = HashSet::default();
            for block in loop_.blocks.iter() {
                let mut instructions_to_keep = Vec::new();
                for instruction_id in inserter.function.dfg[*block].take_instructions() {
                    let mut is_not_loop_invariant = false;
                    // The list of blocks for a nested loop contain any inner loops as well.
                    // We may have already re-inserted new instructions if two loops share blocks
                    // so we need to map all the values in the instruction which we want to check.
                    let (instruction, _) = inserter.map_instruction(instruction_id);
                    instruction.for_each_value(|value| {
                        // We are implicitly checking whether the values are constant as well.
                        // The set of values defined in the loop only contains instruction results and block parameters
                        // which cannot be constants.
                        is_not_loop_invariant |=
                            defined_in_loop.contains(&value) && !loop_invariants.contains(&value);
                    });

                    if !is_not_loop_invariant
                        && instruction.can_be_deduplicated(&inserter.function.dfg, false)
                    {
                        // We need to collect the results as we then mutably borrow to resolve the ValueIds
                        let results =
                            inserter.function.dfg.instruction_results(instruction_id).to_vec();
                        let results = results.into_iter().map(|value| inserter.resolve(value));

                        loop_invariants.extend(results);
                        instructions_to_hoist.push(instruction_id);
                    } else {
                        instructions_to_keep.push(instruction_id);
                    }
                }
                block_to_instructions.insert(*block, instructions_to_keep);
            }

            // Insert instructions we wish to hoist into the pre-header first
            // The loop body and exit are dependent upon these instructions being mapped first
            for instruction_id in instructions_to_hoist {
                let old_results =
                    inserter.function.dfg.instruction_results(instruction_id).to_vec();

                let (instruction, _) = inserter.map_instruction(instruction_id);
                let new_results = push_instruction(
                    instruction_id,
                    instruction,
                    &old_results,
                    pre_header,
                    &mut inserter.function.dfg,
                );

                replace_result_ids(&mut inserter.function.dfg, &old_results, &new_results);
            }

            inserter.map_terminator_in_place(pre_header);

            // Add back and map unchanged loop body instructions
            for (block, instructions_to_keep) in block_to_instructions {
                for instruction_id in instructions_to_keep.iter() {
                    let old_results =
                        inserter.function.dfg.instruction_results(*instruction_id).to_vec();

                    let (instruction, _) = inserter.map_instruction(*instruction_id);
                    let new_results = push_instruction(
                        *instruction_id,
                        instruction,
                        &old_results,
                        block,
                        &mut inserter.function.dfg,
                    );

                    replace_result_ids(&mut inserter.function.dfg, &old_results, &new_results);
                }

                inserter.map_terminator_in_place(block);
            }
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
              v8 = eq v6, u32 6
              constrain v6 == u32 6
              v10 = add v2, u32 1
              jmp b1(v10)
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
            v8 = eq v3, u32 6
            constrain v3 == u32 6
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
            v12 = eq v10, u32 6
            constrain v10 == u32 6
            v13 = add v3, u32 1
            jmp b4(v13)
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
            v12 = eq v4, u32 6
            constrain v4 == u32 6
            v13 = add v3, u32 1
            jmp b4(v13)
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
        println!("{}", ssa);
        assert_normalized_ssa_equals(ssa, expected);
    }
}
