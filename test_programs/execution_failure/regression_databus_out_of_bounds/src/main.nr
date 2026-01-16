pub fn main(
    _x: call_data(1) [u32; 4],
    y: call_data(1) [u32; 4],
    _z: call_data(1) [u32; 4],
    i: u32,
) -> pub u32 {
    y[i]
}
