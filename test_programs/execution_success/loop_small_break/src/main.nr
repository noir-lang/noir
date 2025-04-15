unconstrained fn main(x: Field) {
    let mut count = 0;

    for i in 0..1 {
        if x == 5 {
            count = i;
            break;
        }
    }
    assert(count == 0);
}