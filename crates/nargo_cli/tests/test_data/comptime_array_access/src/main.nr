fn main(a: [Field; 3]) {
    let i = 1;

    // Using a comptime variable as a parameter should not make it non-comptime
    let ii = foo(i);
    let elem1 = a[i];

    // Nor should using it in an expression with a non-comptime variable.
    let two = i + ii;
    constrain i == ii;

    let elem2 = a[i];
    constrain elem1 == elem2;
    constrain two == 2;
}

fn foo(x: Field) -> Field { x }
