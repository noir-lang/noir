fn main() -> pub bool {
    let b: [(bool, &mut u1); 2] = [(false, &mut 0), (true, &mut 1)];

    let x: u32 = 0;
    let y: u32 = 1;
    let z: u32 = 2;

    b[((x - y) % z)].0
}
