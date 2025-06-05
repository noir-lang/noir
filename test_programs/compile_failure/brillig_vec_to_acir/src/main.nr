global DEPTH: u32 = 40000;

fn main(x: [u32; DEPTH], y: u32) {
    let mut new_x = Vec::new();
    new_x = clear(x, y);
}

unconstrained fn clear(x: [u32; DEPTH], y: u32) -> Vec<u32> {
    let mut a = Vec::new();
    for i in 0..y {
        a.push(x[i]);
    }
    a
}
