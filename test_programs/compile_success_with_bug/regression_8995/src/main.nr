fn main() -> pub bool {
    let b: [(bool, &mut bool); 2] = [(false, &mut false), (true, &mut true)];

    let x: u32 = 0;
    let y: u32 = 1;
    let z: u32 = 2;

    b[((x - y) % z)].0
}
