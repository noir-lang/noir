use reexporting_lib::{MyStruct, lib};

mod lib;
use crate::lib::MyStruct;

fn main() {
    let x = MyStruct { inner: 0 };
    assert(lib::is_struct_zero(x));
}
