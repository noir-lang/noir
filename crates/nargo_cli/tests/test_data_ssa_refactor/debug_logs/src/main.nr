use dep::std;

fn main(x : Field, y : pub Field) {

    // std::println_new(x);
    // std::println_new([x, y]);
    
    let s = myStruct { x: x, y: y };
    // std::println_new(s);

    let foo = fooStruct { my_struct: s, foo: 15 };
    // std::println_new(foo);

    // std::println_format("x: {0}, y: {1}", [x, y]);
    
    // TODO: Not supported until we remove aos_to_soa from monomorphization pass
    let s2 = myStruct { x: 20, y: 30 };
    let struct_arr = [s, s2];
    std::println_format("s1: {0}, s2: {1}", [s, s2]);

    assert(x != y);
}

struct myStruct {
    x: Field,
    y: Field,
}

struct fooStruct {
    my_struct: myStruct,
    foo: Field,
}
