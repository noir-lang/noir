struct Foo {
    a: u32,
    b: (bool, [u64; 2]),
    c: [u64; 3],
}

// Printing references to enums don't work at the moment,
// because instead of the tag, we get the address of the tag,
// and we can't tell how many fields follow.
// enum Bar {
//     Baz((u32, [u32; 2])),
// }

unconstrained fn main() {
    show(1, ());
    show(2, 123);
    show(3, [1, 2, 3]);
    show(4, (1, 2, 3));
    show(5, [(1, 2), (3, 4)]);
    show(6, ((1, 2), [3]));
    show(7, "123");
    show(8, (&mut (1, 2), 3));
    show(9, &[1, 2, 3]);
    show(9, &[(1, 2), (3, 4)]);
    show(10, Foo { a: 1, b: (true, [3, 4]), c: [5, 6, 7] });
    // show(11, Bar::Baz((1, [2, 3])));
}

unconstrained fn show<T>(case: u32, mut value: T) {
    // Print a mutable reference and then the same type on its own,
    // to see that data can continue after the reference.
    println((case, &mut value, value))
}
