fn main(mut x: u32, y: u32, z: u32, big_int: U128, hexa: str<7>) {
    let a = U128::from_u64s_le(x as u64, x as u64);
    let b = U128::from_u64s_le(y as u64, x as u64);
    let c = a + b;
    assert(c.lo == z as Field);
    assert(c.hi == 2 * x as Field);
    assert(U128::from_hex(hexa).lo == 0x1f03a);
    let t1 = U128::from_hex("0x9d9c7a87771f03a23783f9d9c7a8777");
    let t2 = U128::from_hex("0x45a26c708BFCF39041");
    let t = t1 + t2;
    assert(t.lo == 0xc5e4b029996e17b8);
    assert(t.hi == 0x09d9c7a87771f07f);
    let t3 = U128::from_le_bytes(t.to_le_bytes());
    assert(t == t3);

    let t4 = t - t2;
    assert(t4 == t1);

    let t5 = U128::from_u64s_le(0, 1);
    let t6 = U128::from_u64s_le(1, 0);
    assert((t5 - t6).hi == 0);

    assert(
        (U128::from_hex("0x71f03a23783f9d9c7a8777") * U128::from_hex("0x8BFCF39041")).hi
            == U128::from_hex("0x3e4e0471b873470e247c824e61445537").hi,
    );
    let q = U128::from_hex("0x3e4e0471b873470e247c824e61445537") / U128::from_hex("0x8BFCF39041");
    assert(q == U128::from_hex("0x71f03a23783f9d9c7a8777"));

    assert(big_int.hi == 2);

    let mut small_int = U128::from_integer(x);
    assert(small_int.lo == x as Field);
    assert(x == small_int.to_integer());
    let shift = small_int << (x as u8);
    assert(shift == U128::from_integer(x << (x as u8)));
    assert(shift >> (x as u8) == small_int);
    assert(shift >> 127 == U128::from_integer(0));
    assert(shift << 127 == U128::from_integer(0));
    assert(U128::from_integer(3).to_integer() == 3);
}

