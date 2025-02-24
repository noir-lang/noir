use reexporting_lib::{FooStruct, lib, MyStruct};

fn main() {
    let x: FooStruct = MyStruct { inner: 0 };
    assert(lib::is_struct_zero(x));
}
