//! This modules defines an SSA pass that inlines calls to simple functions.
//! That is, the contents of the called function is put directly into the caller's body.
//! Functions are still restricted to not be inlined if they are recursive or marked with no predicates.
//!
//! A simple function is defined as the following:
//! - Contains no more than [MAX_INSTRUCTIONS] instructions
//! - The function only has a single block (e.g. no control flow or conditional branches)
//! - It is not marked with the [no predicates inline type][noirc_frontend::monomorphization::ast::InlineType::NoPredicates]
use iter_extended::btree_map;

use crate::ssa::{
    ir::{
        call_graph::CallGraph,
        function::{Function, RuntimeType},
    },
    ssa_gen::Ssa,
};

/// The maximum number of instructions chosen below is an expert estimation of a "small" function
/// in our SSA IR. Generally, inlining small functions with no control flow should enable further optimizations
/// in the compiler while avoiding code size bloat.
///
/// For example, a common "simple" function is writing into a mutable reference.
/// When that function has no control flow, it generally means we can expect all loads and stores within the
/// function to be resolved upon inlining. Inlining this type of basic function both reduces the number of
/// loads/stores to be executed and enables the compiler to continue optimizing at the inline site.
const MAX_INSTRUCTIONS: usize = 10;

impl Ssa {
    /// See the [`inline_simple_functions`][self] module for more information.
    pub(crate) fn inline_simple_functions(mut self: Ssa) -> Ssa {
        let call_graph = CallGraph::from_ssa(&self);
        let recursive_functions = call_graph.get_recursive_functions();

        let should_inline_call = |callee: &Function| {
            if let RuntimeType::Acir(_) = callee.runtime() {
                // Functions marked to not have predicates should be preserved.
                if callee.is_no_predicates() {
                    return false;
                }
            }

            let entry_block_id = callee.entry_block();
            let entry_block = &callee.dfg[entry_block_id];
            let instructions = entry_block.instructions();

            // Only inline functions with a single block
            if entry_block.successors().next().is_some() {
                return false;
            }

            // Only inline functions with `MAX_INSTRUCTIONS` or less instructions
            if instructions.len() > MAX_INSTRUCTIONS {
                return false;
            }

            // Inline zero instructions
            if instructions.is_empty() {
                return true;
            }

            // Check whether the only instruction is a recursive call, which prevents inlining the callee.
            if recursive_functions.contains(&callee.id()) {
                return false;
            }

            true
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

    fn assert_does_not_inline(src: &str) {
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_simple_functions();
        assert_normalized_ssa_equals(ssa, src);
    }

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

        let ssa = ssa.inline_simple_functions();
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

    /// This test is here to make clear that this SSA pass does not attempt multiple passes.
    #[test]
    fn does_not_inline_functions_that_require_multiple_passes() {
        // f2 has greater than 10 instructions, which should prevent it from
        // being inlined into f0 on the first run of this pass.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = call f2(v0) -> Field
            return v1
        }

        acir(inline) fn foo f1 {
          b0(v0: Field):
            return v0
        }

        acir(inline) fn bar f2 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = call f1(v0) -> Field
            v5 = call f1(v0) -> Field
            v6 = call f1(v0) -> Field
            v7 = call f1(v0) -> Field
            v8 = call f1(v0) -> Field
            v9 = call f1(v0) -> Field
            v10 = call f1(v0) -> Field
            v11 = add v1, v2
            return v11
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // In the first pass it won't recognize that `main` could be simplified.
        let mut ssa = ssa.inline_simple_functions();
        assert_ssa_snapshot!(&mut ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f2(v0) -> Field
            return v2
        }
        acir(inline) fn foo f1 {
          b0(v0: Field):
            return v0
        }
        acir(inline) fn bar f2 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
        ");

        // After `bar` has been simplified, it does `main` as well.
        ssa = ssa.inline_simple_functions();
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
        acir(inline) fn bar f2 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
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

        let ssa = ssa.inline_simple_functions();
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
        assert_does_not_inline(src);
    }

    #[test]
    fn does_not_inline_functions_with_no_predicates() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = add v2, v3
            return v4
        }

        acir(no_predicates) fn foo f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ";
        assert_does_not_inline(src);
    }

    #[test]
    fn does_not_inline_function_with_multiple_instructions() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            return v1
        }

        acir(inline) fn foo f1 {
          b0(v0: Field):
            v1 = add v0, Field 1
            v2 = mul v1, Field 2
            v3 = mul v2, Field 2
            v4 = mul v3, Field 2
            v5 = mul v4, Field 2
            v6 = mul v5, Field 2
            v7 = mul v6, Field 2
            v8 = mul v7, Field 2
            v9 = mul v8, Field 2
            v10 = mul v9, Field 2
            v11 = mul v10, Field 2
            return v11
        }
        ";
        assert_does_not_inline(src);
    }

    #[test]
    fn does_not_inline_function_with_multiple_blocks() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: bool):
            v2 = call f1(v0, v1) -> Field
            return v2
        }

        acir(inline) fn foo f1 {
          b0(v0: Field, v1: bool):
            jmpif v1 then: b1, else: b2

          b1():
            v3 = add v0, Field 1
            jmp b3(v3)

          b2():
            v4 = mul v0, Field 2
            jmp b3(v4)

          b3(v5: Field):
            return v5
        }
        ";
        assert_does_not_inline(src);
    }

    #[test]
    fn does_not_inline_mutually_recursive_functions_acir() {
        let src = "
      acir(inline) fn main f0 {
        b0():
          call f1()
          return
      }
      acir(inline) fn starter f1 {
        b0():
          call f2()
          return
      }
      acir(inline) fn main f2 {
        b0():
          call f1()
          return
      }
      ";
        assert_does_not_inline(src);
    }

    #[test]
    fn does_not_inline_mutually_recursive_functions_brillig() {
        let src = "
      acir(inline) fn main f0 {
        b0():
          call f1()
          return
      }
      brillig(inline) fn starter f1 {
        b0():
          call f2()
          return
      }
      brillig(inline) fn ping f2 {
        b0():
          call f3()
          return
      }
      brillig(inline) fn pong f3 {
        b0():
          call f2()
          return
      }
      ";
        assert_does_not_inline(src);
    }
}
