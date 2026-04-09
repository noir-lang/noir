fn main(mut a: bool) -> pub bool {
    if a {
        for _i in 0..1 {
            a = if a { true } else { false };
        }
        true
    } else {
        a
    }
}
