struct WrappedArray<T, N> {
    inner: [T; N]
}

trait MyTrait<T> { }

impl<T, N> MyTrait<N + 1> for WrappedArray<T, N> { }

fn main() { }
