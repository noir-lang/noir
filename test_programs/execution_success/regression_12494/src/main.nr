fn main() -> pub bool {
    let mut hit: bool = false;
    for i in 0..3_u128 {
        if 340282366920938463463374607431768211455 < i {
            hit = true;
        }
    }
    hit
}
