global G_B: u32 = 2825334515;
fn main(b: [[bool; 4]; 4], input: u32) -> pub bool {
    let _a = if b[0][(G_B % 4)] {
        (input * G_B) as u64
    } else {
        3513574538769362461
    };
    false
}
