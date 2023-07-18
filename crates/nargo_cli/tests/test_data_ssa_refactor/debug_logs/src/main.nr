use dep::std;

fn main(x : Field, y : pub Field) {
    // TODO: format strings currently broken unless we assign
    // witness values first
    let i = x;
    let j = y;

    let string: str<14> = "i: {i}, j: {j}";
    std::println_new(string);
    let new_string = "i: {i}, j: {j}";
    std::println_new(new_string);
    let returned_string = string_identity(string);
    std::println_new(returned_string);
    let string_generic = string_with_generic(string);
    std::println_new(string_generic);
    std::println_new("i: {i}, j: {j}");

    let s = myStruct { y: x, x: y };
    std::println_new(s);

    std::println_new("i: {i}, s: {s}");

    std::println_new(x);
    std::println_new([x, y]);
    
    let foo = fooStruct { my_struct: s, foo: 15 };
    std::println_new("s: {s}, foo: {foo}");

    std::println_new("x: 0, y: 1");
    
    let s2 = myStruct { x: 20, y: 30 };
    std::println_new("s1: {s}, s2: {s2}");

    let foo2 = fooStruct { my_struct: s2, foo: 20 };
    std::println_new("foo1: {foo}, foo2: {foo2}");

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
