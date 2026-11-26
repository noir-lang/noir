comptime fn make_u8() -> Quoted {
    quote { 7 as u8 }
}

fn id<T>(a: T) -> T {
    a
}

trait FromWitness {
    fn from_witness(x: Field) -> Self;
}

impl FromWitness for u8 {
    fn from_witness(x: Field) -> u8 {
        x.assert_max_bit_size::<8>();
        x as u8
    }
}

impl FromWitness for Field {
    fn from_witness(x: Field) -> Field {
        x
    }
}

fn main(x: Field) -> pub Field {
    // The second comptime statement fixes `f` to `fn(u8) -> u8`, so the runtime read of `f`
    // has that type and `from_witness` range-checks `x` to 8 bits.
    comptime let f = id;
    comptime { let _ = f(make_u8!()); };
    let g = f;
    let y = g(FromWitness::from_witness(x));
    y as Field
}
