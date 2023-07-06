use dep::std;

fn main() -> pub Field {
    let f = if 3 * 7 > 200 as u32 { foo } else { bar };
    assert(f()[1] == 2);

    // Lambdas:
    assert(twice(|x| x * 2, 5) == 20);
    assert((|x, y| x + y + 1)(2, 3) == 6);

    // Closures:
    let a = 42;
    let g = || a;
    assert(g() == 42);

    // Mutable variables cannot be captured, but you can
    // copy them into immutable variables and capture those:
    let mut x = 2;
    x = x + 1;
    let z = x;

    // Add extra mutations to ensure we can mutate x without the
    // captured z changing.
    x = x + 1;
    assert((|y| y + z)(1) == 4);
    x = x + 1;
    let ret = twice(add1, 3);

    test_array_functions();

    ret
}

/// Test the array functions in std::array
fn test_array_functions() {
    let myarray: [i32; 3] = [1, 2, 3];
    assert(myarray.any(|n| n > 2));

    let evens: [i32; 3] = [2, 4, 6];
    assert(evens.all(|n| n > 1));

    assert(evens.fold(0, |a, b| a + b) == 12);
    assert(evens.reduce(|a, b| a + b) == 12);
    
    let descending = myarray.sort_via(|a, b| a > b);
    assert(descending == [3, 2, 1]);

    assert(evens.map(|n| n / 2) == myarray);
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
