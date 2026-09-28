mod lib {
    pub global N: u32 = 2;
    pub type Size: u32 = N;
}

// `lib::Size` is 2 no matter what the caller names its parameters.
fn main(N: pub u32) {
    assert(N == lib::Size);
}
