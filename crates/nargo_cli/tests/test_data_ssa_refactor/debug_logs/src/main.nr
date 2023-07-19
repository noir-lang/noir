use dep::std;

fn main(x : Field, y : pub Field) {

    std::println_new("*** println_new ***");
    std::println_new(x);
    std::println_new([x, y]);
    
    let s = myStruct { y: x, x: y };
    let foo = fooStruct { my_struct: s, foo: 15 };
    std::println_new(foo);
}

struct myStruct {
    y: Field,
    x: Field,
}

struct fooStruct {
    my_struct: myStruct,
    foo: Field,
}
