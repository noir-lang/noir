use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        function::{Function, RuntimeType},
        function_inserter::FunctionInserter,
        value::ValueId,
    },
    Ssa,
};

use super::{constant_folding::replace_result_ids, unrolling::Loops};

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
            let Ok(unroll_into) = loop_.get_pre_header(&inserter.function, &self.cfg) else {
                continue;
            };

            let mut defined_in_loop: HashSet<ValueId> = HashSet::default();
            for block in loop_.blocks.iter() {
                let params = inserter.function.dfg.block_parameters(*block);
                defined_in_loop.extend(params);
                for instruction_id in inserter.function.dfg[*block].instructions() {
                    let results = inserter.function.dfg.instruction_results(*instruction_id);
                    defined_in_loop.extend(results);
                }
            }

            let mut instructions_to_hoist = Vec::new();
            let mut block_to_instructions = HashMap::default();
            for block in loop_.blocks.iter() {
                let mut instructions_to_keep = Vec::new();
                for instruction_id in inserter.function.dfg[*block].take_instructions() {
                    let mut instr_args_defined_in_loop = false;
                    let instruction = &inserter.function.dfg[instruction_id];
                    instruction.for_each_value(|value| {
                        instr_args_defined_in_loop |= defined_in_loop.contains(&value);
                    });

                    if !instr_args_defined_in_loop
                        && instruction.can_be_deduplicated(&inserter.function.dfg, false)
                    {
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

                if let Some(new_id) = inserter.push_instruction(instruction_id, unroll_into) {
                    let new_results = inserter.function.dfg.instruction_results(new_id).to_vec();
                    replace_result_ids(&mut inserter.function.dfg, &old_results, &new_results);
                }
            }

            // Add back and map unchanged loop body instructions
            for (block, instructions_to_keep) in block_to_instructions {
                for instruction_id in instructions_to_keep.iter() {
                    let old_results =
                        inserter.function.dfg.instruction_results(*instruction_id).to_vec();

                    if let Some(new_id) = inserter.push_instruction(*instruction_id, block) {
                        let new_results =
                            inserter.function.dfg.instruction_results(new_id).to_vec();
                        replace_result_ids(&mut inserter.function.dfg, &old_results, &new_results);
                    }
                }
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
}
