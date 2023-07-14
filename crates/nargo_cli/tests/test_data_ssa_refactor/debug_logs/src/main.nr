use dep::std;

fn main(x : Field, y : pub Field) {
    let i = x;
    let j = y;
    std::println_new("foo1: {i}, foo2: {j}");

    // std::println_new("*** println_new ***");
    // std::println_new(x);
    // std::println_new([x, y]);
    
    // let s = myStruct { y: x, x: y };
    // let foo = fooStruct { my_struct: s, foo: 15 };
    // std::println_new(foo);

    // std::println_new("*** println_format ***");
    // std::println_format("x: {0}, y: {1}", [x, y]);
    
    // let s2 = myStruct { x: 20, y: 30 };
    // std::println_f dormat("s1: {0}, s2: {1}", [s, s2]);

    // let foo2 = fooStruct { my_struct: s2, foo: 20 };
    // std::println_format("foo1: {0}, foo2: {1}", [foo, foo2]);

    assert(x != y);
}

struct myStruct {
    y: Field,
    x: Field,
}

struct fooStruct {
    my_struct: myStruct,
    foo: Field,
}
