use dep::std::bigint;

fn main(mut x: [u8;5], y: [u8;5]) {
    let a = bigint::BigInt::secpk1_fq_from_le_bytes([x[0],x[1],x[2],x[3],x[4]]);
    let b = bigint::BigInt::secpk1_fq_from_le_bytes([y[0],y[1],y[2],y[3],y[4]]);

    let a_bytes = a.to_le_bytes();
    let b_bytes = b.to_le_bytes();
    for i in 0..5 {
        assert(a_bytes[i] == x[i]);
        assert(b_bytes[i] == y[i]);
    }

    let d = a*b - b;
    let d_bytes = d.to_le_bytes();
    let d1 = bigint::BigInt::secpk1_fq_from_le_bytes(597243850900842442924.to_le_bytes(10));
    let d1_bytes = d1.to_le_bytes();
    for i in 0..32 {
        assert(d_bytes[i] == d1_bytes[i]);    
    }
}
