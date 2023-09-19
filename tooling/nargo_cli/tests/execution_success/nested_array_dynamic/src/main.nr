struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field; 3],
    bar: Bar,
}

// fn main(mut x : [Foo; 4], y : pub Field) {
//     dep::std::println(x[3].bar.inner);
// }

fn main(mut x : [Foo; 4], y : pub Field) {
    // assert(x[y - 3].a == 1);
    dep::std::println(x[y - 3].bar.inner);
    dep::std::println(x[y - 2].bar.inner);
    dep::std::println(x[y - 1].bar.inner);
    // assert(x[y - 3].b == [2, 3]);
    // assert(x[y - 2].a == 4);
    // assert(x[y - 2].b == [5, 6]);
    // assert(x[y - 1].a == 7);
    // assert(x[y - 1].b == [8, 9]);
    // assert(x[y].a == 10);
    // assert(x[y].b == [11, 12]);
    // dep::std::println(x[y].bar.inner);
    dep::std::println(x[y].bar.inner);
    assert(x[y].bar.inner == [109, 110, 111]);
    dep::std::println("show list");
    for i in 0..4 {
        dep::std::println(x[i]);
    }
    // if y != 2 {
    //     x[y].a = 50;
    // } else {
    //     x[y].a = 100;
    // }
    // dep::std::println("new list");
    // for i in 0..4 {
    //     dep::std::println(x[i]);
    // }
    // x[y].a = 50;
    // dep::std::println("new list");
    // for i in 0..4 {
    //     dep::std::println(x[i]);
    // }
    // assert(x[y].a == 50);

    // if y == 2 {
    //     x[y - 1].b = [50, 51];
    // } else {
    //     x[y - 1].b = [100, 101];
    // }
    // x[y - 1].b = [100, 101];
    // dep::std::println("new list");
    // for i in 0..4 {
    //     dep::std::println(x[i]);
    // }
    // dep::std::println(x[2].b);
    // assert(x[2].b == [100, 101]);
}

