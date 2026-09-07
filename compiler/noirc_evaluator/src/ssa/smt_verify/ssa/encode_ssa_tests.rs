//! Snapshot tests for `encode_to_smt_text` — pinning the literal SMT-LIB2
//! translation of each SSA instruction and construction the `Encoder`
//! currently handles.

use super::encode_to_smt_text;

#[test]
fn parameter_is_returned_directly() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "acir(inline) fn main f0 {
               b0(v0: Field):
                 return v0
             }",
        ),
        @r"
    (declare-const p0 FF)
    return: p0
    "
    );
}

#[test]
fn constant_is_returned_directly() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "acir(inline) fn main f0 {
               b0(v0: Field):
                 return Field 5
             }",
        ),
        @r"
    (declare-const p0 FF)
    return: (as ff5 FF)
    "
    );
}

#[test]
fn add_binary_instruction() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "acir(inline) fn main f0 {
               b0(v0: Field):
                 v1 = add v0, v0
                 return v1
             }",
        ),
        @r"
    (declare-const p0 FF)
    (declare-const t_1 FF)
    (assert (= t_1 (ff.add p0 p0)))
    return: t_1
    "
    );
}

#[test]
fn sub_binary_instruction() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "acir(inline) fn main f0 {
               b0(v0: Field):
                 v1 = sub v0, v0
                 return v1
             }",
        ),
        @r"
    (declare-const p0 FF)
    (declare-const t_1 FF)
    (assert (= t_1 (ff.add p0 (ff.neg p0))))
    return: t_1
    "
    );
}

#[test]
fn mul_binary_instruction() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "acir(inline) fn main f0 {
               b0(v0: Field):
                 v1 = mul v0, v0
                 return v1
             }",
        ),
        @r"
    (declare-const p0 FF)
    (declare-const t_1 FF)
    (assert (= t_1 (ff.mul p0 p0)))
    return: t_1
    "
    );
}

#[test]
fn constant_operand_is_inlined_not_declared() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "acir(inline) fn main f0 {
               b0(v0: Field):
                 v1 = add v0, Field 1
                 return v1
             }",
        ),
        @r"
    (declare-const p0 FF)
    (declare-const t_1 FF)
    (assert (= t_1 (ff.add p0 (as ff1 FF))))
    return: t_1
    "
    );
}

#[test]
fn shared_value_is_declared_once() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "acir(inline) fn main f0 {
               b0(v0: Field):
                 v1 = add v0, v0
                 v2 = add v1, v1
                 v3 = add v2, v2
                 return v3
             }",
        ),
        @r"
    (declare-const p0 FF)
    (declare-const t_1 FF)
    (assert (= t_1 (ff.add p0 p0)))
    (declare-const t_2 FF)
    (assert (= t_2 (ff.add t_1 t_1)))
    (declare-const t_3 FF)
    (assert (= t_3 (ff.add t_2 t_2)))
    return: t_3
    "
    );
}

#[test]
fn multiple_return_values() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "acir(inline) fn main f0 {
               b0(v0: Field, v1: Field):
                 v2 = add v0, v1
                 return v2, v0
             }",
        ),
        @r"
    (declare-const p0 FF)
    (declare-const p1 FF)
    (declare-const t_2 FF)
    (assert (= t_2 (ff.add p0 p1)))
    return: t_2, p0
    "
    );
}
