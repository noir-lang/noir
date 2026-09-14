// Two `#[fold]` functions with different signatures, first called in the reverse of their
// declaration order. Each fold circuit must be paired with its own signature regardless of
// the order in which `main`'s body first references the functions.
#[fold]
fn fold_one(x: Field) -> Field {
    x + 1
}

#[fold]
fn fold_two(x: Field, y: Field) -> Field {
    x + y
}

fn main(x: Field, y: pub Field) {
    let a = fold_two(x, y);
    let b = fold_one(x);
    assert(a == x + y);
    assert(b == x + 1);
}
