fn main() {
    bug();
    comptime { bug() };
}

fn bug() {
    let mut a = ([1, 2], 3);
    a.0[{
    a = ([4, 5], 6);
    1
    }] = 7;

    assert_eq(a, ([4, 7], 6));
}
