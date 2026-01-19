unconstrained fn main() -> pub () {
    foo(true, 1);
}

unconstrained fn foo(a: bool, mut b: Field) -> Field {
    if a {
        0
    } else {
        for i in 0..(if ((b as u128) != (b as u128)) { 0 } else { 0 }) {
            foo(true, 1);
            b = (-1 / if a { 1 } else { -1 });
        }
        a as Field
    }
}
