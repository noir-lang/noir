// Test integer addition: 3 + 4 = 7
use dep::std::uint128::U128;
use dep::std;

fn main(mut x: u32, y: u32, z: u32, hexa : str<7>) {
    let a = U128 {
        lo: x as Field,
        hi: x as Field,
    };
    let b = U128 {
        lo: y as Field,
        hi: x as Field,
    };
    let c = a.add(b);
    assert(c.lo == z as Field);
    assert(c.hi == 2*x as Field);
    assert(U128::from_hex(hexa).lo == 0x1f03a);
    let t1 =U128::from_hex("0x9d9c7a87771f03a23783f9d9c7a8777");
    let t2 = U128::from_hex("0x45a26c708BFCF39041");
    let t = t1.add(t2);
    assert(t.lo == 0xc5e4b029996e17b8);
    assert(t.hi == 0x09d9c7a87771f07f);
    let t3 = U128::from_le_bytes(t.to_le_bytes());
    assert(t.lo == t3.lo);
    assert(t.hi == t3.hi);
}

