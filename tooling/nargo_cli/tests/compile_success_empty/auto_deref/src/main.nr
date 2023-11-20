fn main() {
    let a = &mut &mut &mut [1, 2, 3];
    assert(a[0] == 1);

    a[0] = 4;
    assert(a[0] == 4);
}
