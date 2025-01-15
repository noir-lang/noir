
// 3 values
struct Bar {
    inner: [Field; 3],
}

// 7 values
struct Foo {
    a: Field,
    b: [Field; 3],
    bar: Bar,
}

// 10 values
struct FooParent {
    array: [Field; 3],
    foos: [Foo; 1],
}

fn main(mut x: [Foo; 1], y: pub Field) {
    // if y == 3 {
    //     x[y - 3].b[y - 1] = 5000;
    // }
    // assert(x[y - 3].b[y - 1] == 5000);

    let foo_parent_one = FooParent { array: [0, 1, 2], foos: x };
    let foo_parent_two = FooParent { array: [3, 4, 5], foos: x };
    let mut foo_parents = [foo_parent_one, foo_parent_two];

    if y == 3 {
        foo_parents[y - 2].foos[y - 3].b[y - 1] = 5000;
    } 
    assert(foo_parents[1].foos[0].b[2] == 5000);
}

// fn main(mut x: [Foo; 1], y: pub Field) {
//     let mut foo_parents = [0; 20];
//     foo_parents = [
//         0, 1, 2, x[0].a, x[0].b[0], x[0].b[1], x[0].b[2], x[0].bar.inner[0], x[0].bar.inner[1], x[0].bar.inner[2],
//         3, 4, 5, x[0].a, x[0].b[0], x[0].b[1], x[0].b[2], x[0].bar.inner[0], x[0].bar.inner[1], x[0].bar.inner[2]
//     ];

//     let index_foo_parent = ((y - 2) * 10);
//     // Add 3 to skip `array` in FooParent
//     let index_foos = index_foo_parent + 3;
//     // Add 1 to skip `a` in Foo. 
//     let index_foo_b = index_foos + 3;
//     if y == 3 {
//         foo_parents[index_foo_b] = 5000;
//     } else {
//         foo_parents[index_foo_b] = 1000;
//     }
//     assert(foo_parents[16] == 5000);
// }
