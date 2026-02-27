fn main() -> pub bool {
    let b: [(bool, &mut bool); 1] = [(false, &mut false)];
    b[1].0
}
