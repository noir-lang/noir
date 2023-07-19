use dep::std;

fn main(x : Field, y : pub Field) {

    std::println("*** println ***");
    std::println(x);
    std::println([x, y]);
    
    let s = myStruct { y: x, x: y };
    let foo = fooStruct { my_struct: s, foo: 15 };
    std::println(foo);
}

struct myStruct {
    y: Field,
    x: Field,
}

struct fooStruct {
    my_struct: myStruct,
    foo: Field,
}
