fn main() -> pub bool {
    foo().0
}

fn foo() -> (bool, i8) {
    (true, (-128_i8 / -1_i8))
}
