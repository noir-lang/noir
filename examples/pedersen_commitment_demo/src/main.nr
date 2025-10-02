// Pedersen commitment demo: commit = H(value, blinding)

fn commit(value: Field, blinding: Field) -> Field {
    std::hash::pedersen_hash([value, blinding])
}

// Circuit checks: given (value, blinding), recompute commit
// and expose it as public output.
fn main(value: Field, blinding: Field) -> pub Field {
    commit(value, blinding)
}

#[test]
fn test_commitment_consistency() {
    let v: Field = 123;
    let r: Field = 999;
    let c = commit(v, r);
    let recomputed = main(v, r);
    assert(c == recomputed);
}
