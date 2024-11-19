use std::collections::HashSet;

use crate::ssa::{
    ir::{function::RuntimeType, instruction::Instruction, value::Value},
    Ssa,
};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_unused_brillig_functions(mut self) -> Ssa {
        // Compute the set of all brillig functions that exist in the program
        let mut brillig_function_ids = HashSet::new();
        for (func_id, func) in &self.functions {
            if let RuntimeType::Brillig(..) = func.runtime() {
                brillig_function_ids.insert(*func_id);
            };
        }

        // Remove from the above set functions that are called
        for function in self.functions.values() {
            for block_id in function.reachable_blocks() {
                for instruction_id in function.dfg[block_id].instructions() {
                    let instruction = &function.dfg[*instruction_id];
                    let Instruction::Call { func: func_id, arguments: _ } = instruction else {
                        continue;
                    };

                    let func_value = &function.dfg[*func_id];
                    let Value::Function(func_id) = func_value else { continue };

                    brillig_function_ids.remove(func_id);
                }
            }
        }

        // The ones that remain are never called: let's remove them.
        for func_id in brillig_function_ids {
            // We never want to remove the main function (it could be `unconstrained` or it
            // could have been turned into brillig if `--force-brillig` was given).
            // We also don't want to remove entry points.
            if self.main_id == func_id || self.entry_point_to_generated_index.contains_key(&func_id)
            {
                continue;
            }

            self.functions.remove(&func_id);
        }
        self
    }
}

#[cfg(test)]
mod test {
    use crate::ssa::opt::assert_normalized_ssa_equals;

    use super::Ssa;

    #[test]
    fn removes_unused_brillig_functions() {
        // In the SSA below the function `two` is never called so we expected it to be removed.
        let src = "
            acir(inline) fn main f0 {
              b0():
                call f1()
                return
            }

            brillig(inline) fn one f1 {
              b0():
                return
            }

            brillig(inline) fn two f2 {
              b0():
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0():
                call f1()
                return
            }
            brillig(inline) fn one f1 {
              b0():
                return
            }
            ";
        let ssa = ssa.remove_unused_brillig_functions();
        assert_normalized_ssa_equals(ssa, expected);
    }
}
