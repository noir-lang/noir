fn main() -> pub Field {
    let mut d: [&mut bool; 1] = [(&mut false)];

    for _ in 0..1 {} // Without this it compiles

    if *d[0] {
        0
    } else {
        1
    }
}
