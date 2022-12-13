fn main() -> pub Field {
    let f = if 3 * 7 > 200 { foo } else { bar };
    constrain f()[1] == 2;

    // lambdas!
    constrain twice(|x| x * 2, 5) == 20;
    constrain (|x, y| x + y + 1)(2, 3) == 6;

    twice(add1, 3)
}

fn foo() -> [u32; 2] {
    [1, 3]
}

fn bar() -> [u32; 2] {
    [3, 2]
}

fn add1(x: Field) -> Field {
    x + 1
}

fn twice(f: fn(Field) -> Field, x: Field) -> Field {
    f(f(x))
}
