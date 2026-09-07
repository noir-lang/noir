//! Snapshot tests for `encode_to_smt_text` — pinning the literal SMT-LIB2
//! translation of each ACIR opcode the encoder currently handles.

use super::encode_to_smt_text;

#[test]
fn assert_zero_linear() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "
            private parameters: [w0, w1]
            public parameters: []
            return values: []
            ASSERT w0 = w1
            ",
        ),
        @r"
    (define-fun zero () FF (as ff0 FF))
    (declare-const w0 FF)
    (declare-const w1 FF)
    (assert (= (ff.add w0 (ff.neg w1)) zero))
    "
    );
}

#[test]
fn assert_zero_with_mul_term() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "
            private parameters: [w0, w1, w2]
            public parameters: []
            return values: []
            ASSERT w2 = w0*w1
            ",
        ),
        @r"
    (define-fun zero () FF (as ff0 FF))
    (declare-const w0 FF)
    (declare-const w1 FF)
    (declare-const w2 FF)
    (assert (= (ff.add (ff.neg (ff.mul w0 w1)) w2) zero))
    "
    );
}

/// Confirms `one` is only defined when actually needed: a constant of `-1`
/// still ends up using it (via `(ff.neg one)`), even though `-1` isn't `1`.
#[test]
fn assert_zero_defines_one_only_when_needed() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "
            private parameters: [w0, w1]
            public parameters: []
            return values: []
            ASSERT w1 = w0 + 1
            ",
        ),
        @r"
    (define-fun zero () FF (as ff0 FF))
    (define-fun one () FF (as ff1 FF))
    (declare-const w0 FF)
    (declare-const w1 FF)
    (assert (= (ff.add w1 (ff.neg w0) (ff.neg one)) zero))
    "
    );
}

#[test]
fn assert_zero_with_coefficient_and_constant() {
    insta::assert_snapshot!(
        encode_to_smt_text(
            "
            private parameters: [w0, w1]
            public parameters: []
            return values: []
            ASSERT w1 = 5*w0 + 2
            ",
        ),
        @r"
    (define-fun zero () FF (as ff0 FF))
    (declare-const w0 FF)
    (declare-const w1 FF)
    (assert (= (ff.add w1 (ff.mul (ff.neg (as ff5 FF)) w0) (ff.neg (as ff2 FF))) zero))
    "
    );
}
