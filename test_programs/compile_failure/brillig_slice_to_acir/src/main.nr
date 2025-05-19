global DEPTH: u32 = 40000;

fn main(x: [u32; DEPTH], y: u32) {
    let mut new_x = [];
    new_x = clear(x, y);
}

unconstrained fn clear(x: [u32; DEPTH], y: u32) -> [u32] {
    let mut a = [];
    for i in 0..y {
        a = a.push_back(x[i]);
    }
    a
}
