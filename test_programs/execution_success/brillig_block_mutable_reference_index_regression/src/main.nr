fn main(arr: [u32; 3], idx: u32) -> pub [u32; 3] {
    let x: [[u32; 3]; 1] = [arr];
    let z: &mut [u32; 3] = &mut { x[0] };
    let y = x;
    (*z)[idx] = 100;
    y[0]
}
