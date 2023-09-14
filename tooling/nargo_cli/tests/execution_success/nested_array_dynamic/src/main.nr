
struct Foo {
    a: Field,
    b: Field,
}

fn main(mut x : [Foo; 3], y : pub Field) {
    assert(x[y - 2].a == 1);
    assert(x[y - 2].b == 2);
    assert(x[y - 1].a == 3);
    assert(x[y - 1].b == 4);
    assert(x[y].a == 5);
    assert(x[y].b == 6);

    if y != 2 {
        x[y].a = 50;
    } else {
        x[y].a = 100;
    }
    
    dep::std::println(x);
}
