fn main() -> pub bool {
    let b: [&mut bool] = #[(&mut true), (&mut false)];
    *b[2]
}
