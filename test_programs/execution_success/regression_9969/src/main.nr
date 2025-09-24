fn main(b: bool) -> pub Field {
    let a: [&mut Field] = &[];
    if b {
        *a[0]
    } else {
        0
    }
}
