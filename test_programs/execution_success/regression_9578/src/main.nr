struct Foo {
    a: u32,
    b: (bool, [u64; 2]),
    c: [u64; 3],
}

// A value containing a reference can be printed: the compiler loads every reference before the
// `print` oracle call, so the oracle receives a reference-free value. References to an enum work
// too, because once the reference is loaded the tag is a concrete value (rather than the address
// of the tag, which is why this used to be unsupported).
//
// References are loaded through tuples, structs and fixed-size arrays, so a reference nested inside
// any of those can also be printed. References nested inside an enum variant or a vector are still
// not supported: the compiler cannot load them ahead of the call without knowing the active variant
// or a dynamic length.
enum Bar {
    Baz((u32, [u32; 2])),
}

struct WithRef {
    a: &mut u32,
    b: u32,
}

unconstrained fn main() {
    show(1, ());
    show(2, 123);
    show(3, [1, 2, 3]);
    show(4, (1, 2, 3));
    show(5, [(1, 2), (3, 4)]);
    show(6, ((1, 2), [3]));
    show(7, "123");
    show(8, (&mut (1, 2), 3));
    show(9, [1, 2, 3].as_vector());
    show(9, [(1, 2), (3, 4)].as_vector());
    show(10, Foo { a: 1, b: (true, [3, 4]), c: [5, 6, 7] });
    show(11, Bar::Baz((1, [2, 3])));

    let mut x = 12;
    let mut y = 13;
    show(14, WithRef { a: &mut x, b: 15 });
    show(16, [&mut x, &mut y]);
}

unconstrained fn show<T>(case: u32, mut value: T) {
    // Print a mutable reference and then the same type on its own,
    // to see that data can continue after the reference.
    println((case, &mut value, value))
}
