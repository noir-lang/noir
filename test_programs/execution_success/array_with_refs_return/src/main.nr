// Regression for issue #8786 (https://github.com/noir-lang/noir/issues/8786)
unconstrained fn main() -> pub [i8; 2] {
    if (*func_5()[1].0) {
        [0, 1]
    } else {
        [2, 3]
    }
}
unconstrained fn func_5() -> [(&mut bool, str<2>, str<4>, str<4>); 2] {
    [((&mut true), "AB", "WTGS", "AELL"), ((&mut false), "CD", "PKHQ", "LDWE")]
}
