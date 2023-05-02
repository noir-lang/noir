fn main(x : bool, y: [bool;2]) {
    if x {
        assert(1 != 2);
    }

    assert(x);
    assert(y[0] != y[1]);
}
