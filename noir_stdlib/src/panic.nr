pub fn panic<T, U>(message: T) -> U {
    assert(false, message);
    crate::mem::zeroed()
}
