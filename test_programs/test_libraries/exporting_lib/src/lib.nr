pub struct MyStruct {
    pub inner: Field
}

pub type FooStruct = MyStruct;

pub fn is_struct_zero(val: MyStruct) -> bool {
    val.inner == 0
}
