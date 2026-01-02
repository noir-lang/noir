fn main(v0: i8, cond: bool) {
    if cond {
        let v4 = v0 / -1;
        let v5 = v0 < v4;
        assert(v5);
    }
    if cond {
        let v8 = v0 / -1;
        let v9 = v0 < v8;
        assert(v9);
    }
}
