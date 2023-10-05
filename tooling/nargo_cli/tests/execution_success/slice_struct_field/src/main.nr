use dep::std::collections::vec::Vec;

struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field],
    bar: Bar,
}

// fn main(y : pub Field) {
//     let b_one = [2, 3, 20];
//     let foo_one = Foo { a: 1, b: b_one, bar: Bar { inner: [100, 101, 102] } };
//     let b_two = [5, 6, 21];
//     let foo_two = Foo { a: 4, b: b_two, bar: Bar { inner: [103, 104, 105] } };
//     let foo_three = Foo { a: 7, b: [8, 9, 22], bar: Bar { inner: [106, 107, 108] } };
//     let foo_four = Foo { a: 10, b: [11, 12, 23], bar: Bar { inner: [109, 110, 111] } };

//     let mut x = [foo_one, foo_two];
//     x = x.push_back(foo_three);
//     x = x.push_back(foo_four);

//     dep::std::println(x[y].bar.inner);
// }

// NOTE: reorg does nothing cause we just read the whole struct when we
// have slices in the struct
// struct FooReorg {
//     a: Field,
//     bar: Bar,
//     b: [Field],
// }

// struct FooParent {
//     parent_arr: [Field; 3],
//     foos: [Foo],
// }

// struct Foo {
//     a: Field,
//     b: [Field; 3],
//     bar: Bar,
// }

// struct Foo {
//     a: Field,
//     b: Vec<Field>,
//     bar: Bar,
// }

fn main(y : pub Field) {
    let mut b_one = [2, 3, 20];
    b_one = b_one.push_back(20);
    // let b_one = Vec::from_slice([2, 3, 20]);
    let foo_one = Foo { a: 1, b: b_one, bar: Bar { inner: [100, 101, 102] } };
    let mut b_two = [5, 6, 21];
    b_two = b_two.push_back(21);
    // let b_two = Vec::from_slice([5, 6, 21]);
    let foo_two = Foo { a: 4, b: b_two, bar: Bar { inner: [103, 104, 105] } };
    let foo_three = Foo { a: 7, b: [8, 9, 22], bar: Bar { inner: [106, 107, 108] } };
    let foo_four = Foo { a: 10, b: [11, 12, 23], bar: Bar { inner: [109, 110, 111] } };

    let mut x = [foo_one, foo_two];
    x = x.push_back(foo_three);
    x = x.push_back(foo_four);
    
    // assert(x[y - 3].a == 1);
    // let struct_slice = x[y - 3].b;
    // for i in 0..4 {
    //     assert(struct_slice[i] == b_one[i]);
    // }

    // assert(x[y - 2].a == 4);
    // let struct_slice = x[y - 2].b;
    // for i in 0..4 {
    //     assert(struct_slice[i] == b_two[i]);
    // }

    // assert(x[y - 1].a == 7);
    // let struct_slice = x[y - 1].b;
    // assert(struct_slice[0] == 8);
    // assert(struct_slice[1] == 9);
    // assert(struct_slice[2] == 22);

    // assert(x[y].a == 10);
    // let struct_slice = x[y].b;
    // assert(struct_slice[0] == 11);
    // assert(struct_slice[1] == 12);
    // assert(struct_slice[2] == 23);
    // assert(x[y].bar.inner == [109, 110, 111]);

    // Check dynamic array set 
    if y != 2 {
        // dep::std::println("inside if case");
        // dep::std::println(x[y].a);
        x[y].a = 50;
        // dep::std::println(x[y].a);
    } else {
        x[y].a = 100;
    }
    assert(x[3].a == 50);

    // dep::std::println(x[3].a);
    // TODO: get this assert to pass

    // assert(struct_slice())

    // for i in 0..4 {
    //     dep::std::println(x[i].a);
    //     if i < 2 as u64 {
    //         let struct_slice = x[i].b;
    //         for i in 0..4 {
    //             dep::std::println(struct_slice[i]);

    //         }
    //     } else {
    //         let struct_slice = x[i].b;
    //         for i in 0..3 {
    //             dep::std::println(struct_slice[i]);
    //         } 
    //     }
    //     dep::std::println(x[i].bar.inner);
    // }

    // dep::std::println(x[y - 3].a);
    // dep::std::println(x[y - 3].bar.inner);
    // dep::std::println(x[y - 2].a);
    // dep::std::println(x[y - 2].bar.inner);
    // dep::std::println(x[y - 1].a);
    // dep::std::println(x[y - 1].bar.inner);
    // dep::std::println(x[y].a);
    // dep::std::println(x[y].bar.inner);

    // assert(x[y].a == 10);
    // assert(x[y].b == [11, 12, 23]);
    // assert(x[y].bar.inner == [109, 110, 111]);
}