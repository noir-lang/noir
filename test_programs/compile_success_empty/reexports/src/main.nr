use dep::reexporting_lib::{FooStruct, MyStruct, lib};

fn main() {
    let x: FooStruct = MyStruct { inner: 0 };
    assert(lib::is_struct_zero(x));
}
