//! Rejects calls to intrinsics that the calling function's runtime cannot lower.
//!
//! `recursive_aggregation` exists only as circuit constraints, so Brillig has no lowering for it,
//! and `field_less_than` exists only for unconstrained execution, so ACIR has none. Both back ends
//! treat the other runtime's intrinsic as unreachable.
//!
//! Whether a function is constrained is decided at monomorphization, and code is commonly guarded
//! with `is_unconstrained()` so that each runtime only reaches the calls it can lower. The check
//! therefore runs on the SSA that is about to be lowered, after the passes that fold
//! `is_unconstrained()` and remove the branches it disables, and reports the call that survives
//! with its call stack.
use acvm::acir::BlackBoxFunc;

use crate::{
    errors::RuntimeError,
    ssa::{
        ir::{
            instruction::{Instruction, Intrinsic},
            value::Value,
        },
        ssa_gen::Ssa,
    },
};

impl Ssa {
    /// Returns an error for the first call to an intrinsic that the calling function's runtime
    /// cannot lower.
    pub(crate) fn check_runtime_only_intrinsics(&self) -> Result<(), RuntimeError> {
        for function in self.functions.values() {
            let is_brillig = function.runtime().is_brillig();
            for block_id in function.reachable_blocks() {
                for instruction_id in function.dfg[block_id].instructions() {
                    let Instruction::Call { func, .. } = &function.dfg[*instruction_id] else {
                        continue;
                    };
                    let Value::Intrinsic(intrinsic) = &function.dfg[*func] else {
                        continue;
                    };
                    let call_stack = || function.dfg.get_instruction_call_stack(*instruction_id);
                    match intrinsic {
                        Intrinsic::BlackBox(BlackBoxFunc::RecursiveAggregation) if is_brillig => {
                            return Err(RuntimeError::RecursiveAggregationInUnconstrained {
                                call_stack: call_stack(),
                            });
                        }
                        Intrinsic::FieldLessThan if !is_brillig => {
                            return Err(RuntimeError::UnconstrainedOnlyIntrinsicInConstrained {
                                name: intrinsic.to_string(),
                                call_stack: call_stack(),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{errors::RuntimeError, ssa::ssa_gen::Ssa};

    fn check(src: &str) -> Result<(), RuntimeError> {
        Ssa::from_str(src).unwrap().check_runtime_only_intrinsics()
    }

    #[test]
    fn rejects_recursive_aggregation_in_brillig() {
        let src = r#"
            brillig(inline) predicate_pure fn main f0 {
              b0(v0: u32):
                v1 = make_array [Field 0] : [Field; 1]
                v2 = make_array [Field 0] : [Field; 1]
                v3 = make_array [Field 0] : [Field; 1]
                call recursive_aggregation(v1, v2, v3, Field 0, u32 0)
                return
            }
        "#;
        assert!(matches!(
            check(src),
            Err(RuntimeError::RecursiveAggregationInUnconstrained { .. })
        ));
    }

    #[test]
    fn accepts_recursive_aggregation_in_acir() {
        let src = r#"
            acir(inline) predicate_pure fn main f0 {
              b0(v0: u32):
                v1 = make_array [Field 0] : [Field; 1]
                v2 = make_array [Field 0] : [Field; 1]
                v3 = make_array [Field 0] : [Field; 1]
                call recursive_aggregation(v1, v2, v3, Field 0, u32 0)
                return
            }
        "#;
        assert!(check(src).is_ok());
    }

    #[test]
    fn rejects_field_less_than_in_acir() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: Field, v1: Field):
                v2 = call field_less_than(v0, v1) -> u1
                return v2
            }
        "#;
        assert!(matches!(
            check(src),
            Err(RuntimeError::UnconstrainedOnlyIntrinsicInConstrained { name, .. }) if name == "field_less_than"
        ));
    }

    #[test]
    fn accepts_field_less_than_in_brillig() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0(v0: Field, v1: Field):
                v2 = call field_less_than(v0, v1) -> u1
                return v2
            }
        "#;
        assert!(check(src).is_ok());
    }

    #[test]
    fn ignores_calls_in_unreachable_blocks() {
        // Once `is_unconstrained()` is folded, the branch it disables is unreachable; a call left
        // there is never lowered and is not reported.
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                return
              b1():
                v1 = make_array [Field 0] : [Field; 1]
                call recursive_aggregation(v1, v1, v1, Field 0, u32 0)
                return
            }
        "#;
        assert!(check(src).is_ok());
    }
}
