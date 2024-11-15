pub global TWO_POW_128: Field = 0x100000000000000000000000000000000;

unconstrained fn decompose_hint(x: Field) -> (Field, Field) {
    (0, 0)
}

fn decompose(x: Field) -> (Field, Field) {
    unsafe {
        let (xlo, xhi) = decompose_hint(x);

        xlo.assert_max_bit_size::<128>();
        xhi.assert_max_bit_size::<128>();
        //assert_eq(x, xlo + TWO_POW_128 * xhi);
        (xlo, xhi)
    }
}

pub fn main(x: Field) -> pub (Field, Field) {
    decompose(x)
}
