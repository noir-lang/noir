struct Foo {
    a: u32,
    b: (bool, [u64; 2]),
    c: [u64; 3],
}

// Printing references is not supported, because Oracles cannot handle 
// references in parameters.
// #9578 has been superseded by #12838

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
}

unconstrained fn show<T>(case: u32, mut value: T) {
    // Print a mutable reference and then the same type on its own,
    // to see that data can continue after the reference.
    println((case, &mut value, value))
}
