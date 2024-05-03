fn main(x: bool, mut y: [u32; 30]) {
    if x {
        y[0] = 1;
    }

    let z = y[0] + y[1];
    assert(z == 1);
}
