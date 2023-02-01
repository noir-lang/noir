fn main() -> pub Field {
    let f = if 3 * 7 > 200 { foo } else { bar };
    constrain f()[1] == 2;

    // Lambdas:
    constrain twice(|x| x * 2, 5) == 20;
    constrain (|x, y| x + y + 1)(2, 3) == 6;

    // Closures:
    let a = 42;
    let g = || a;
    constrain g() == 42;

    // Mutable variables cannot be captured, but you can
    // copy them into immutable variables and capture those:
    let mut x = 2;
    x = x + 1;
    let z = x;

    // Add extra mutations to ensure we can mutate x without the
    // captured z changing.
    x = x + 1;
    constrain (|y| y + z)(1) == 4;
    x = x + 1;

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
