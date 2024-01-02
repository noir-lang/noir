struct MyStruct {
    foo: bool,
    bar: [str<5>; 3],
}

struct NestedStruct {
    foo: MyStruct,
    bar: [MyStruct; 3],
    baz: u64
}

fn main(
    x: u64,
    y: pub u64,
    array: [u8; 5],
    my_struct: NestedStruct,
    string: str<5>
) -> pub (u64, u64, MyStruct) {
    assert(array.len() == 5);
    assert(my_struct.foo.foo);
    assert(string == "12345");

    assert(x < y);
    (x + y, 3, my_struct.foo)
}
