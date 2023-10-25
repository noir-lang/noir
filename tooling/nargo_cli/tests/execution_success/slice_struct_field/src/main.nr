use dep::std::println;

struct FooParent {
    parent_arr: [Field; 3],
    foos: [Foo],
}

struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field],
    bar: Bar,
}

fn main(y : pub Field) {
    let mut b_one = [2, 3, 20];
    b_one = b_one.push_back(20);
    let foo_one = Foo { a: 1, b: b_one, bar: Bar { inner: [100, 101, 102] } };
    
    let mut b_two = [5, 6, 21];
    b_two = b_two.push_back(21);
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

    // assert(x[y - 3].bar.inner == [100, 101, 102]);
    // assert(x[y - 2].bar.inner == [103, 104, 105]);
    // assert(x[y - 1].bar.inner == [106, 107, 108]);
    // assert(x[y].bar.inner == [109, 110, 111]);

    // TODO: Enable merging nested slices 
    // if y != 2 {
    //     x[y].a = 50;
    // } else {
    //     x[y].a = 100;
    // }
    // assert(x[3].a == 50);

    // if y == 2 {
    //     x[y - 1].b = [50, 51, 52];
    // } else {
    //     x[y - 1].b = [100, 101, 102];
    // }
    // assert(x[2].b[0] == 100);
    // assert(x[2].b[1] == 101);
    // assert(x[2].b[2] == 102);

    let q = x.push_back(foo_four);
    let foo_parent_one = FooParent { parent_arr: [0, 1, 2], foos: x };
    let foo_parent_two = FooParent { parent_arr: [3, 4, 5], foos: q };
    let mut foo_parents = [foo_parent_one];
    foo_parents = foo_parents.push_back(foo_parent_two);

    // TODO: Merging nested slices is broken
    // if y == 3 {
    //     foo_parents[y - 2].foos[y - 1].b[y - 1] = 5000;
    // } else {
    //     foo_parents[y - 2].foos[y - 1].b[y - 1] = 1000;
    // }
    // assert(foo_parents[y - 2].foos[y - 1].a == 7);

    // assert(foo_parents[y - 2].foos[y - 2].b[y - 1] == 21);
    // foo_parents[y - 2].foos[y - 2].b[y - 1] = 5000;
    // assert(foo_parents[y - 2].foos[y - 2].b[y - 1] == 5000);

    // let b_array = foo_parents[y - 2].foos[y - 3].b;
    // assert(foo_parents[y - 2].foos[y - 3].a == 1);
    // assert(b_array[0] == 2);
    // assert(b_array[1] == 3);
    // assert(b_array[2] == 20);
    // assert(b_array[3] == 20);

    // let b_array = foo_parents[y - 2].foos[y - 2].b;
    // assert(foo_parents[y - 2].foos[y - 2].a == 4);
    // assert(b_array[0] == 5);
    // assert(b_array[1] == 6);
    // assert(b_array[2] == 5000);
    // assert(b_array[3] == 21);

    // assert(foo_parents[y - 2].foos[y - 1].a == 7);
    // foo_parents[y - 2].foos[y - 1].a = 50;
    // assert(foo_parents[y - 2].foos[y - 1].a == 50);

    // let b_array = foo_parents[y - 2].foos[y - 1].b;
    // assert(b_array[0] == 8);
    // assert(b_array[1] == 9);
    // assert(b_array[2] == 22);
    // assert(b_array.len() == 3);

    let mut new_foo_one_b = foo_parents[y - 2].foos[y - 2].b;
    // foo_parents[y - 2].foos[y - 1].b = new_foo_one_b;
    // new_foo_one_b = new_foo_one_b.push_back(100);
    let x = [5, 6, 5000, 21, 100, 101].as_slice();
    foo_parents[y - 2].foos[y - 1].b = x;

    assert(foo_parents[y - 2].foos[y - 1].b.len() == 6);
    // dep::std::println(foo_parents[y - 2].foos[y - 1].b[4]);
    assert(foo_parents[y - 2].foos[y - 1].b[4] == 100);
    assert(foo_parents[y - 2].foos[y - 1].b[5] == 101);

    // TODO: Slice intrinsics on dynamic nested slices needs special handling in ACIR gen
    foo_parents[y - 2].foos[y - 1].b = foo_parents[y - 2].foos[y - 1].b.push_back(500);
    println(foo_parents[y - 2].foos[y - 1].b.len());
    assert(foo_parents[y - 2].foos[y - 1].b[6] == 500);
    
    let b_array = foo_parents[y - 2].foos[y - 1].b;
    assert(b_array[0] == 5);
    assert(b_array[1] == 6);
    assert(b_array[2] == 5000);
    assert(b_array[3] == 21);

//     let b_array = foo_parents[y - 2].foos[y].b;
//     assert(foo_parents[y - 2].foos[y].a == 10);
//     assert(b_array[0] == 11);
//     assert(b_array[1] == 12);
//     assert(b_array[2] == 23);

//     assert(foo_parents[y - 2].foos[y - 3].bar.inner == [100, 101, 102]);
//     assert(foo_parents[y - 2].foos[y - 2].bar.inner == [103, 104, 105]);
//     assert(foo_parents[y - 2].foos[y - 1].bar.inner == [106, 107, 108]);
//     assert(foo_parents[y - 2].foos[y].bar.inner == [109, 110, 111]);
}