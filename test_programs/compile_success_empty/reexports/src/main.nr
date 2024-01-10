use dep::reexporting_lib::{FooStruct, MyStruct, is_struct_zero};

fn main() {
    let x: FooStruct = MyStruct {
        inner: 0
    };
    assert(is_struct_zero(x));
}
