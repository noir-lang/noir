struct MyStruct {
    foo: bool,
    bar: [str<5>; 3],
    baz: Field
}

struct NestedStruct {
    foo: MyStruct,
    bar: [MyStruct; 3],
    baz: u64
}

#[export]
fn exported_function_foo(x: u64, y: u64, array: [u8; 5], my_struct: NestedStruct, string: str<5>) -> (u64, u64, MyStruct) {
    assert(array.len() == 5);
    assert(my_struct.foo.foo);
    assert(string == "12345");

    print(x);
    assert(x < y);
    (x + y, 3, my_struct.foo)
}

#[export]
fn exported_function_bar(my_struct: NestedStruct) -> (u64) {
    my_struct.baz
}
