struct MyStruct<let STR_LEN: u32, let BAR_SIZE: u32> {
    foo: bool,
    bar: [str<STR_LEN>; BAR_SIZE],
    baz: Field
}

struct NestedStruct<let BAR_SIZE: u32, let STR_LEN: u32, let NESTED_BAR_SIZE: u32, BAZ_TYP> {
    foo: MyStruct<STR_LEN, NESTED_BAR_SIZE>,
    bar: [MyStruct<STR_LEN, NESTED_BAR_SIZE>; BAR_SIZE],
    baz: BAZ_TYP
}

#[export]
fn exported_function_foo(
    x: u64,
    y: u64,
    array: [u8; 5],
    my_struct: NestedStruct<2,3,4, Field>,
    string: str<5>
) -> (u64, u32, MyStruct<3, 4>) {
    assert(array.len() == 5);
    assert(my_struct.foo.foo);
    assert(string == "12345");

    print(x);
    assert(x < y);
    (x + y, 3, my_struct.foo)
}

#[export]
fn exported_function_bar(my_struct: NestedStruct<1,2,3, u64>) -> (u64) {
    my_struct.baz
}
