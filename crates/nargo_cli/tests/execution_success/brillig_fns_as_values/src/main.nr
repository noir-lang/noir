struct MyStruct {
    operation: fn (u32) -> u32,
}

fn main(x: u32)  {
    assert_eq(wrapper(increment, x), x + 1);
    assert_eq(wrapper(increment_acir, x), x + 1);
    assert_eq(wrapper(decrement, x), x - 1);
    assert_eq(wrapper_with_struct(MyStruct { operation: increment }, x), x + 1);
    assert_eq(wrapper_with_struct(MyStruct { operation: decrement }, x), x - 1);
    // https://github.com/noir-lang/noir/issues/1975
    assert_eq(increment(x), x + 1);
}

unconstrained fn wrapper(func: fn (u32) -> u32, param: u32) -> u32 {
    func(param)
}

unconstrained fn increment(x: u32) -> u32 {
    x + 1
}

unconstrained fn decrement(x: u32) -> u32 {
    x - 1
}

unconstrained fn wrapper_with_struct(my_struct: MyStruct, param: u32) -> u32 {
    let func = my_struct.operation;
    func(param)
}



fn increment_acir(x: u32) -> u32 {
    x + 1
}
