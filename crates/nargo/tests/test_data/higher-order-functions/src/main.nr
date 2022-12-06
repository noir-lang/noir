fn main() {
    let f = if 3 * 7 > 200 { foo } else { bar };
    constrain f()[1] == 2;
}

fn foo() -> [u32; 2] {
    [1, 3]
}

fn bar() -> [u32; 2] {
    [3, 2]
}