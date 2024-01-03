use dep::std;

fn main(mut x: u32, y: u32, z: u32, big_int: u128, hexa : str<7>) {
    let a = u128::new(x as u64, x as u64);
    let b = u128::new(y as u64, x as u64);
    let c = a.add(b);
    assert(c.lo == z as Field);
    assert(c.hi == 2*x as Field);
    assert(u128::from_hex(hexa).lo == 0x1f03a);
    let t1 =u128::from_hex("0x9d9c7a87771f03a23783f9d9c7a8777");
    let t2 = u128::from_hex("0x45a26c708BFCF39041");
    let t = t1.add(t2);
    assert(t.lo == 0xc5e4b029996e17b8);
    assert(t.hi == 0x09d9c7a87771f07f);
    let t3 = u128::from_le_bytes(t.to_le_bytes());
    assert(t.eq(t3));

    let t4 = t.sub(t2);
    assert(t4.eq(t1));

    let t5 = u128::new(0,1);
    let t6 = u128::new(1,0);
    assert(t5.sub(t6).hi == 0);

    assert(u128::from_hex("0x71f03a23783f9d9c7a8777").mul(u128::from_hex("0x8BFCF39041")).hi == u128::from_hex("0x3e4e0471b873470e247c824e61445537").hi);
    let q = u128::from_hex("0x3e4e0471b873470e247c824e61445537").div(u128::from_hex("0x8BFCF39041"));
    assert(q.eq(u128::from_hex("0x71f03a23783f9d9c7a8777")));

    assert(big_int.hi == 2);

    let small_int = u128::from_integer(x);
    assert(small_int.lo == x as Field);
    assert(x == small_int.to_integer());
}

