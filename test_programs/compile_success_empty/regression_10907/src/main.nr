pub struct PrivateCall<let N: u32> {
    pub args: [Field; N],
}

impl<let N: u32> PrivateCall<N> {
    pub fn new(args: [Field; N]) -> Self {
        Self { args }
    }
}

fn main() {
    let _ = PrivateCall::<_>::new([0, 0]);
}
