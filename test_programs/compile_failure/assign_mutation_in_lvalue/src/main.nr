fn main() {
    bug();
    comptime { bug() };
}

fn bug() {
    let mut a = ([1, 2], 3);
    a.0[{
        a = ([4,5], 6);
        1
    }] = 7;

    // acir+brillig yield ([1, 7], 3)
    // comptime yields ([1, 7], 6)
    // both should yield:
    assert_eq(a, ([4, 7], 6));
}
