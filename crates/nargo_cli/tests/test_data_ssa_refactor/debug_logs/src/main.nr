use dep::std;

fn main(x : Field, y : pub Field) {
    let i = x;
    let j = y;
    let s = myStruct { y: x, x: y };
    std::println_new(s);

    std::println_new("foo1: {i}, foo2: {s}");

    std::println_new(x);
    std::println_new([x, y]);
    
    let foo = fooStruct { my_struct: s, foo: 15 };
    std::println_new("s: {s}, foo: {foo}");

    std::println_new("x: 0, y: 1");
    
    let s2 = myStruct { x: 20, y: 30 };
    std::println_new("s1: {s}, s2: {s2}");

    let foo2 = fooStruct { my_struct: s2, foo: 20 };
    std::println_new("foo1: {foo}, foo2: {foo2}");

    // assert(x != y);
}

struct myStruct {
    y: Field,
    x: Field,
}

struct fooStruct {
    my_struct: myStruct,
    foo: Field,
}
