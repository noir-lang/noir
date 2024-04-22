struct MyStruct {
    inner: field
}

type FooStruct = MyStruct;

fn is_struct_zero(val: MyStruct) -> bool {
    val.inner == 0
}
