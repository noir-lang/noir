
struct Foo {
    a: Field,
    b: [Field; 2],
}

fn main(mut x : [Foo; 4], y : pub Field) {
    dep::std::println(x[y - 3]);
    dep::std::println(x[y - 2]);
    dep::std::println(x[y - 1]);
    dep::std::println(x[y]);

    assert(x[y - 3].a == 1);
    assert(x[y - 3].b == [2, 3]);
    assert(x[y - 2].a == 4);
    assert(x[y - 2].b == [5, 6]);
    assert(x[y - 1].a == 7);
    assert(x[y - 1].b == [8, 9]);
    assert(x[y].a == 10);
    assert(x[y].b == [11, 12]);

    // if y != 2 {
    //     x[y].a = 50;
    // } else {
    //     x[y].a = 100;
    // }
    // assert(x[y].a == 50);

    // dep::std::println(x[y].a);
}

// fn main(mut x : [Foo; 4], y : pub Field) {
//     dep::std::println(x[0]);
//     dep::std::println(x[1]);
//     dep::std::println(x[2]);
//     dep::std::println(x[3]);
// }
