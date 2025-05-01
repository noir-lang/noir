//! This modules defines an SSA pass that inlines calls to functions with at most one instruction.
//! That is, the contents of the called function is put directly into the caller's body.
//! Functions are still restricted to not be inlined if they are recursive or marked with no predicates.
use iter_extended::btree_map;

use crate::ssa::{
    ir::{
        function::{Function, RuntimeType},
        instruction::Instruction,
        value::Value,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See the [`inline_functions_with_at_most_one_instruction`][self] module for more information.
    pub(crate) fn inline_functions_with_at_most_one_instruction(mut self: Ssa) -> Ssa {
        let should_inline_call = |callee: &Function| {
            if let RuntimeType::Acir(_) = callee.runtime() {
                // Functions marked to not have predicates should be preserved.
                if callee.is_no_predicates() {
                    return false;
                }
            }

            let entry_block_id = callee.entry_block();
            let entry_block = &callee.dfg[entry_block_id];

            // Only inline functions with a single block
            if entry_block.successors().next().is_some() {
                return false;
            }

            // Only inline functions with 0 or 1 instructions
            if entry_block.instructions().len() > 1 {
                return false;
            }

            let instructions = callee.dfg[entry_block_id].instructions();
            if instructions.is_empty() {
                return true;
            }

            // Check whether the only instruction is a recursive call, which prevents inlining the callee.
            // This special check is done here to avoid performing the entire inline info computation.
            // The inline info computation contains extra logic and requires passing over every function.
            // which we can avoid in when inlining simple functions.
            let only_instruction = callee.dfg[entry_block_id].instructions()[0];
            if let Instruction::Call { func, .. } = callee.dfg[only_instruction] {
                let Value::Function(func_id) = callee.dfg[func] else {
                    return true;
                };

                func_id != callee.id()
            } else {
                true
            }
        };

        self.functions = btree_map(&self.functions, |(id, function)| {
            (*id, function.inlined(&self, &should_inline_call))
        });

        self
    }
}

#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        ssa::{Ssa, opt::assert_normalized_ssa_equals},
    };

    #[test]
    fn inline_functions_with_zero_instructions() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = add v2, v3
            return v4
        }

        acir(inline) fn foo f1 {
          b0(v0: Field):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.inline_functions_with_at_most_one_instruction();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
        acir(inline) fn foo f1 {
          b0(v0: Field):
            return v0
        }
        ");
    }

    #[test]
    fn inline_functions_with_one_instruction() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = add v2, v3
            return v4
        }

        acir(inline) fn foo f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.inline_functions_with_at_most_one_instruction();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = add v0, Field 1
            v3 = add v0, Field 1
            v4 = add v2, v3
            return v4
        }
        acir(inline) fn foo f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ");
    }

    #[test]
    fn does_not_inline_function_with_one_instruction_that_calls_itself() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            return v1
        }

        acir(inline) fn foo f1 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.inline_functions_with_at_most_one_instruction();
        assert_normalized_ssa_equals(ssa, src);
    }
}
