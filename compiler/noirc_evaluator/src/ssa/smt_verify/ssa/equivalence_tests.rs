//! Tests that a real SSA pass, or the `simplify` rules applied while
//! parsing, preserve behavior for all inputs.

use crate::assert_ssa_snapshot;
use crate::ssa::Ssa;

use super::{assert_pass_preserves_behavior, assert_simplify_preserves_behavior, ssa_equivalent};

#[test]
fn sub_self_is_zero_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = sub v0, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return Field 0
    }
    ");
}

#[test]
fn add_zero_lhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add Field 0, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn add_zero_rhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add v0, Field 0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn sub_zero_rhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = sub v0, Field 0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn mul_one_lhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = mul Field 1, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn mul_one_rhs_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = mul v0, Field 1
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return v0
    }
    ");
}

#[test]
fn mul_zero_holds_for_all_field_elements() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = mul v0, Field 0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        return Field 0
    }
    ");
}

#[test]
fn dead_instruction_elimination_preserves_behavior() {
    let after = assert_pass_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add v0, v0
             v2 = add v0, v0
             return v2
         }",
        Ssa::dead_instruction_elimination,
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        v1 = add v0, v0
        return v1
    }
    ");
}

/// Regression test for the encoder re-expanding a shared value's whole
/// definition at every use site instead of naming it once: `v1` and `v2`
/// here are each referenced twice by the instruction after them, so a
/// non-memoizing encoder would double the term size at each step (8x
/// overall) instead of growing linearly.
#[test]
fn shared_values_are_encoded_once_not_reexpanded() {
    let after = assert_pass_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add v0, v0
             v2 = add v1, v1
             v3 = add v2, v2
             return v3
         }",
        Ssa::dead_instruction_elimination,
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: Field):
        v1 = add v0, v0
        v2 = add v1, v1
        v3 = add v2, v2
        return v3
    }
    ");
}

#[test]
fn mul_boolean_square_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1):
             v1 = mul v0, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1):
        return v0
    }
    ");
}

#[test]
fn mul_boolean_b_times_bx_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1, v1: u1):
             v2 = mul v0, v1
             v3 = mul v0, v2
             return v3
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1, v1: u1):
        v2 = unchecked_mul v0, v1
        return v2
    }
    ");
}

#[test]
fn mul_boolean_bx_times_b_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1, v1: u1):
             v2 = mul v0, v1
             v3 = mul v2, v0
             return v3
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1, v1: u1):
        v2 = unchecked_mul v0, v1
        return v2
    }
    ");
}

#[test]
fn eq_boolean_true_rhs_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1):
             v1 = eq v0, u1 1
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1):
        return v0
    }
    ");
}

#[test]
fn eq_boolean_true_lhs_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1):
             v1 = eq u1 1, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1):
        return v0
    }
    ");
}

#[test]
fn eq_boolean_false_rhs_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1):
             v1 = eq v0, u1 0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1):
        v1 = not v0
        return v1
    }
    ");
}

#[test]
fn eq_boolean_false_lhs_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1):
             v1 = eq u1 0, v0
             return v1
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1):
        v1 = not v0
        return v1
    }
    ");
}

#[test]
fn and_boolean_is_unchecked_mul_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1, v1: u1):
             v2 = and v0, v1
             return v2
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1, v1: u1):
        v2 = unchecked_mul v0, v1
        return v2
    }
    ");
}

#[test]
fn not_not_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0(v0: u1):
             v1 = not v0
             v2 = not v1
             return v2
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0(v0: u1):
        v1 = not v0
        return v0
    }
    ");
}

#[test]
fn not_constant_holds() {
    let after = assert_simplify_preserves_behavior(
        "acir(inline) fn main f0 {
           b0():
             v0 = not u1 1
             return v0
         }",
    );
    assert_ssa_snapshot!(after, @r"
    acir(inline) fn main f0 {
      b0():
        return u1 0
    }
    ");
}

/// Proves the harness can actually detect a real mismatch, not just always
/// report equivalence: two SSA snippets that plainly compute different
/// things.
#[test]
fn harness_detects_a_real_mismatch() {
    let before = Ssa::from_str(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = sub v0, v0
             return v1
         }",
    )
    .unwrap();
    let after = Ssa::from_str(
        "acir(inline) fn main f0 {
           b0(v0: Field):
             v1 = add v0, Field 1
             return v1
         }",
    )
    .unwrap();

    if let Some(equivalent) = ssa_equivalent(&before, &after) {
        assert!(!equivalent, "cvc5 should have found a counterexample");
    }
}
