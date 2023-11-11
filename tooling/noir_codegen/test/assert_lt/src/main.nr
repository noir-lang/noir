struct MyStruct {
    foo: bool,
    bar: [str<5>; 3],
}

fn main(x: u64, y: pub u64, array: [u8; 5], my_struct: MyStruct) -> pub u64 {
    assert(array.len() == 5);
    assert(my_struct.foo);

    assert(x < y);
    x + y
}
