struct Bar {
    inner: [Field; 3],
}

fn main(y: pub u32) {
    let bar = Bar { inner: [100, 101, 102] };
    if y < 10 {
        assert(bar.inner ==  [100, 101, 102]);
    }
}
