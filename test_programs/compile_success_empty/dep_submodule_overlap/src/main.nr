use reexporting_lib::{FooStruct, MyStruct, lib};

mod lib;
use crate::lib::{FooStruct, MyStruct};

fn main() {
    let x: FooStruct = MyStruct { inner: 0 };
    assert(lib::is_struct_zero(x));
}

