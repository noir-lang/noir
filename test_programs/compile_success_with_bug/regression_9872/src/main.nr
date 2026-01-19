fn main() -> pub bool {
    let b: [&mut bool] = [(&mut true), (&mut false)].as_vector();
    *b[2]
}
