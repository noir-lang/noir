global G_A: [[bool; 0]; 2] = [[], []];

fn main(a: bool, b: [(u1, bool, u1, str<2>); 4]) -> pub [bool; 0] {
    if (a) {
        G_A[((((b[0].0 as u8) + (b[0].0 as u8)) as u32) % 2)]
    } else {
        G_A[((((b[0].0 as u8) + (b[0].0 as u8)) as u32) % 2)]
    }
}
