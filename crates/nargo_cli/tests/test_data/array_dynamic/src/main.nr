
fn main(x: [u32; 5], mut z: u32, t: u32) {
    let idx = (z - 5*t - 5) as Field;
    //dynamic array test
    dyn_array(x, idx, idx - 3); 
}

fn dyn_array(mut x: [u32; 5], y: Field, z: Field) {
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