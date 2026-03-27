fn main(a: u128) {
    let f: [(&mut str<0>, u128); 2] = [(&mut "", 1), (&mut "", a)];
    for i in 0..2 {
        assert(f[i].1 <= 1);
    }
}
