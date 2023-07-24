use dep::std;

fn main(x : Field, y : pub Field) {
    // TODO: format strings currently broken unless we assign
    // witness values first
    let i = x;
    let j = y;

    let string: str<14> = "i: {i}, j: {j}";
    std::println(string);
    let string_no_type = "i: {i}, j: {j}";
    std::println(string_no_type);
    let returned_string = string_identity(string_no_type);
    std::println(returned_string);
    let string_generic = string_with_generic(string_no_type);
    std::println(string_generic);
    std::println("i: {i}, j: {j}");

    let s = myStruct { y: x, x: y };
    std::println(s);

    std::println("i: {i}, s: {s}");

    std::println(x);
    std::println([x, y]);
    
    let foo = fooStruct { my_struct: s, foo: 15 };
    std::println("s: {s}, foo: {foo}");

    std::println("x: 0, y: 1");
    
    let s2 = myStruct { x: 20, y: 30 };
    std::println("s1: {s}, s2: {s2}");

    let foo2 = fooStruct { my_struct: s2, foo: 20 };
    std::println("foo1: {foo}, foo2: {foo2}");

    assert(x != y);
}

fn string_identity(string: str<14>) -> str<14> {
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
