fn main(x: Field, y: Field, a: Field) {
    let z = (x - y) * a;
    z.assert_max_bit_size::<4>();
    assert(a * a == 1);
}