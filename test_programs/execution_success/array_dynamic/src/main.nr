fn main(
    x: [u32; 5],
    mut z: u32,
    t: u32,
    index: [u32; 5],
    index2: [u32; 5],
    offset: u32,
    sublen: Field,
) {
    let idx = (z - 5 * t - 5);
    //dynamic array test
    dyn_array(x, idx, idx - 3);
    //regression for issue 1283
    let mut s = 0;
    let x3 = [246, 159, 32, 176, 8];
    for i in 0..5 {
        s += x3[index[i]];
    }
    assert(s != 0);

    if 3 < (sublen as u32) {
        assert(index[offset + 3] == index2[3]);
    }
}

fn dyn_array(mut x: [u32; 5], y: u32, z: u32) {
    assert(x[y] == 111);
    assert(x[z] == 101);
    x[z] = 0;
    assert(x[y] == 111);
    assert(x[1] == 0);
    if y as u32 < 10 {
        x[y] = x[y] - 2;
    } else {
        x[y] = 0;
    }
    assert(x[4] == 109);
}
