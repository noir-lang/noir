use dep::std;

fn main() -> pub Field {
    let f = if 3 * 7 > 200 as u32 { foo } else { bar };
    assert(f()[1] == 2);
    // Lambdas:
    assert(twice(|x| x * 2, 5) == 20);
    assert((|x, y| x + y + 1)(2, 3) == 6);

    // nested lambdas
    assert((|a, b| {
        a + (|c| c + 2)(b)
    })(0, 1) == 3);


    // Closures:
    let a = 42;
    let g = || a;
    assert(g() == 42);

    // When you copy mutable variables,
    // the capture of the copies shouldn't change:
    let mut x = 2;
    x = x + 1;
    let z = x;

    // Add extra mutations to ensure we can mutate x without the
    // captured z changing.
    x = x + 1;
    assert((|y| y + z)(1) == 4);

    // When you capture mutable variables,
    // again, the captured variable doesn't change: 
    let closure_capturing_mutable = (|y| y + x);
    assert(closure_capturing_mutable(1) == 5);
    x += 1;
    assert(closure_capturing_mutable(1) == 5);

    // Fixing an ICE, where rewriting the closures
    // during monomorphization didn't correspond
    // to an internal `if` type
    // found by @jfecher:
    // https://github.com/noir-lang/noir/pull/1959#issuecomment-1658992989
    let x2: u32 = 32;

    let closure_if_else = if x2 > 2 {
        || x2
    } else {
        || x2 + 2342
    };

    assert(closure_if_else() == 32);

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

    // TODO: is this a sort_via issue with the new backend,
    // or something more general?
    //
    // currently it fails only with `--experimental-ssa` with
    //  "not yet implemented: Cast into signed"
    // but it worked with the original ssa backend
    // (before dropping it)
    //
    // opened #2121 for it
    // https://github.com/noir-lang/noir/issues/2121

    // let descending = myarray.sort_via(|a, b| a > b);
    // assert(descending == [3, 2, 1]);

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
