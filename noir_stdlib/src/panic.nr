pub fn panic<T, U>(message: T) -> U
where
    T: StringLike,
{
    assert(false, message);
    crate::mem::zeroed()
}

trait StringLike {}

impl<let N: u32> StringLike for str<N> {}
impl<let N: u32, T> StringLike for fmtstr<N, T> {}
