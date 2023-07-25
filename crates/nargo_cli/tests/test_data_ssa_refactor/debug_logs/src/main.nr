use dep::std;

fn main(x : Field, y : pub Field) {
    let string = "i: {i}, j: {j}";
    std::println(string);

    // A `fmtstr` lets you easily perform string interpolation.
    // You can 
    let fmt_str: fmtstr<14, (Field, Field)> = f"i: {x}, j: {y}";
    let fmt_str = string_identity(fmt_str);
    std::println(fmt_str);

    let fmt_str_no_type = f"i: {x}, j: {y}";
    std::println(fmt_str_no_type);

    let fmt_str_generic = string_with_generic(fmt_str_no_type);
    std::println(fmt_str_generic);

    let s = myStruct { y: x, x: y };
    std::println(s);

    std::println(f"i: {x}, s: {s}");

    std::println(x);
    std::println([x, y]);
    
    let foo = fooStruct { my_struct: s, foo: 15 };
    std::println(f"s: {s}, foo: {foo}");

    std::println(f"x: 0, y: 1");
    
    let s2 = myStruct { x: 20, y: 30 };
    std::println(f"s1: {s}, s2: {s2}");

    let foo2 = fooStruct { my_struct: s2, foo: 20 };
    std::println(f"foo1: {foo}, foo2: {foo2}");

    assert(x != y);
}

fn string_identity(string: fmtstr<14, (Field, Field)>) -> fmtstr<14, (Field, Field)> {
    string
}

fn string_with_generic<T>(string: T) -> T {
    string
}

struct myStruct {
    y: Field,
    x: Field,
}

struct fooStruct {
    my_struct: myStruct,
    foo: Field,
}
