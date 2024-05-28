trait MyTrait<T> { }

struct WrappedArray<T, N> {
    inner: [T; N]
}

// expected to fail with:
// error: impl's are not allowed on generic arithmetic
impl<T, N> MyTrait<N> for WrappedArray<T, N + 1> { }

fn main() { }
