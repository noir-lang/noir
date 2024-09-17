// #[oracle(avm_calldata)]
pub fn avm_read_calldata<T>() -> T {
    panic(f"avm built-in")
}

// #[oracle(avm_return)]
pub fn avm_return<T>(_: T) {
    panic(f"avm built-in")
}
