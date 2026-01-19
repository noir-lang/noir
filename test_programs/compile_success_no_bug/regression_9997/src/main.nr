fn main(a: pub u32) -> pub Field {
    if a == 0 {
        let c: [&mut Field] = [(&mut 1)].as_vector();
        1 / { (*c[3]) }
    } else {
        2
    }
}
