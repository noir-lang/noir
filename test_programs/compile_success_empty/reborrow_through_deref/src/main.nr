struct Pair {
    a: Field,
    b: Field,
}

fn main() {
    let mut pair = Pair { a: 1, b: 2 };
    let pair_ref = &mut pair;
    let field_ref = &mut (*pair_ref).b;
    *field_ref = 9;
    assert(pair.b == 9);
}
