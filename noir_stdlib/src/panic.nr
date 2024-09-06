pub fn panic<T, U, let N: u32>(message: fmtstr<N, T>) -> U {
    assert(false, message);
    std::mem::zeroed()
}
