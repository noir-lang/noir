use std::biguint::BigUint;

fn main() {
    test_from_le_bytes1();
    test_from_le_bytes2();
    test_from_le_bytes_32();
    test_to_le_blocks_and_sub();
    test_to_le_bytes_32();

    test_add1();
    test_add2();
    test_adc();
    test_sub1();
    test_sub2();
    test_sbb();

    test_mul1();
    test_mul2();
    test_cmp1();
    test_cmp2();

    test_shl1();
    test_shl2();
    test_shl3();
    test_shl4();
    test_shl5();
    test_shl6();

    test_msb1();
    test_msb2();
    test_msb3();
    test_msb4();

    test_div1();
    test_div2();
    test_div3();
    test_div4();
    test_div5();

    test_sqrt1();
    test_sqrt2();
    test_sqrt3();
    test_sqrt4();
    test_sqrt5();
    test_sqrt6();
}

fn test_from_le_bytes1() {
    let a = BigUint::from_le_bytes(&[2]);

    assert(
        a.eq(
            BigUint {
        blocks: [
            2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    }
        )
    );
}

fn test_from_le_bytes2() {
    // 7 bytes of 0xff
    let a = BigUint::from_le_bytes(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    assert(
        a.eq(
            BigUint {
        blocks: [
            0xffffffff, 0xffffff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    }
        )
    );
}

fn test_from_le_bytes_32() {
    let actual = BigUint::from_le_bytes_32([0xff; 32]);
    assert(
        actual.eq(
            BigUint {
        blocks: [
            0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
            0xffffffff, 0xffffffff, 0xffffffff, 0, 0, 0, 0,
        ]
    }
        )
    );
}

fn test_to_le_blocks_and_sub() {
    let a: BigUint<12> = BigUint::zero();
    let b: BigUint<12> = BigUint::one();

    let c = a.sub(b);
    assert(c.to_le_blocks() == [0xffffffff; 12]);
}

fn test_to_le_bytes_32() {
    let a: BigUint<12> = BigUint::from_le_bytes_32([
        0, 158, 104, 228, 242, 160, 170, 77,
        4, 139, 81, 3, 106, 231, 27, 83,
        220, 149, 200, 117, 204, 253, 208, 19,
        168, 149, 181, 163, 93, 182, 151, 88
    ]);

    let expected_bytes = [
        0, 158, 104, 228, 242, 160, 170, 77,
        4, 139, 81, 3, 106, 231, 27, 83,
        220, 149, 200, 117, 204, 253, 208, 19,
        168, 149, 181, 163, 93, 182, 151, 88
    ];
    assert(a.to_le_bytes_32() == expected_bytes);
}

fn test_add1() {
    let a: BigUint<12> = BigUint::from_le_bytes(&[1]);
    let b: BigUint<12> = BigUint::from_le_bytes(&[2]);
    let sum = a.add(b);

    assert(sum.eq(BigUint::from_le_bytes(&[3])));
}

fn test_add2() {
    let a: BigUint<12> = BigUint::from_le_bytes(&[0xff, 0xff, 0xff, 0xff]);
    let b: BigUint<12> = BigUint::from_le_bytes(&[1]);
    let sum = a.add(b);

    assert(sum.eq(BigUint::from_le_bytes(&[0, 0, 0, 0, 1])));
}

fn test_adc() {
    let a: BigUint<12> = BigUint { blocks: [0xffffffff; 12] };
    let b: BigUint<12> = BigUint::one();
    let (sum, carry) = a.adc(b);

    assert(sum.eq(BigUint::zero()));
    assert(carry == 1);
}

fn test_sub1() {
    let a: BigUint<12> = BigUint::from_le_bytes(&[5]);
    let b: BigUint<12> = BigUint::from_le_bytes(&[3]);
    let diff = a.sub(b);

    assert(diff.eq(BigUint::from_le_bytes(&[2])));
}

fn test_sub2() {
    let a: BigUint<12> = BigUint::from_le_bytes(&[1, 2]);
    let b: BigUint<12> = BigUint::from_le_bytes(&[2]);
    let diff = a.sub(b);

    assert(diff.eq(BigUint::from_le_bytes(&[0xff, 1])));
}

fn test_sbb() {
    let a: BigUint<12> = BigUint::from_le_bytes(&[1]);
    let b: BigUint<12> = BigUint::from_le_bytes(&[2]);
    let (diff, borrow) = a.sbb(b);

    assert(diff.eq(BigUint::from_le_bytes(&[0xff; 4*12])));
    assert(borrow >> (32 - 1) as u8 == 1);
}

fn test_mul1() {
    let a: BigUint<12> = BigUint::from_le_bytes(&[2]);
    let b: BigUint<12> = BigUint::from_le_bytes(&[3]);
    let (lo, hi) = a.mul(b);
    
    assert(lo.eq(BigUint::from_le_bytes(&[6])));
    assert(hi.eq(BigUint::zero()));
}

fn test_mul2() {
    let a = BigUint { blocks: [0xffffffff; 12] };
    let b = BigUint { blocks: [0xffffffff; 12] };
    let (lo, hi) = a.mul(b);

    let mut expected_hi = [0xffffffff; 12];
    expected_hi[0] = 0xfffffffe;

    assert(lo.eq(BigUint::one()));
    assert(hi.to_le_blocks() == expected_hi);
}

fn test_cmp1() {
    let a: BigUint<5> = BigUint::from_le_bytes(&[2, 0]);
    let b: BigUint<5> = BigUint::from_le_bytes(&[0, 2]);

    assert(!a.eq(b));
    assert(!a.gte(b));

    assert(a.lt(b));
    assert(!a.gt(b));
}

fn test_cmp2() {
    let a: BigUint<11> = BigUint::from_le_bytes(&[0, 2]);
    let b: BigUint<11> = BigUint::from_le_bytes(&[0, 2]);

    assert(a.eq(b));
    assert(a.gte(b));

    assert(!a.lt(b));
    assert(!a.gt(b));
}

fn test_shl1() {
    let a: BigUint<15> = BigUint::from_le_bytes(&[1, 2]);
    let b = a.shl(0);

    assert(a.eq(b));
}

fn test_shl2() {
    let a: BigUint<12> = BigUint::from_le_bytes(&[1, 2]);
    let b: BigUint<12> = a.shl(32*12);

    assert(b.eq(BigUint::zero()));
}

fn test_shl3() {
    let a: BigUint<8> = BigUint::from_le_bytes(&[1, 2]);
    let b = a.shl(1);

    let blocks = b.to_le_blocks();
    let mut expected = [0 as u32; 8];
    expected[0] = 2 + 4*256;

    assert(blocks == expected);
}

fn test_shl4() {
    let a: BigUint<5> = BigUint::from_le_bytes(&[1, 2]);
    let b: BigUint<5> = a.shl(8);
    let expected: BigUint<5> = BigUint::from_le_bytes(&[0, 1, 2]);

    assert(b.eq(expected));
}

fn test_shl5() {
    let a: BigUint<5> = BigUint::from_le_bytes(&[1]);
    let b: BigUint<5> = a.shl1();
    let expected: BigUint<5> = BigUint::from_le_bytes(&[2]);

    assert(b.eq(expected));
}

fn test_shl6() {
    let a: BigUint<10> = BigUint::from_le_bytes(&[0x80]);
    let b: BigUint<10> = a.shl1();
    let expected: BigUint<10> = BigUint::from_le_bytes(&[0, 1]);

    assert(b.eq(expected));
}

fn test_msb1() {
    let a: BigUint<10> = BigUint::from_le_bytes(&[0]);
    let b = a.msb();

    assert(b == 0);
}

fn test_msb2() {
    let a: BigUint<7> = BigUint::from_le_bytes(&[1]);
    let b = a.msb();

    assert(b == 1);
}

fn test_msb3() {
    let a: BigUint<4> = BigUint::from_le_bytes(&[0xff]);
    let b = a.msb();

    assert(b == 8);
}

fn test_msb4() {
    let a: BigUint<2> = BigUint::from_le_bytes(&[1, 5]);
    let b = a.msb();

    assert(b == 11);
}

fn test_div1() {
    let a: BigUint<2> = BigUint::from_le_bytes(&[7]);
    let b: BigUint<2> = BigUint::from_le_bytes(&[3]);

    let (q, r) = a.div(b);
    let expected_q: BigUint<2> = BigUint::from_le_bytes(&[2]);
    let expected_r: BigUint<2> = BigUint::from_le_bytes(&[1]);

    assert(q.eq(expected_q));
    assert(r.eq(expected_r));
}

fn test_div2() {
    let a: BigUint<4> = BigUint::from_le_bytes(&[7, 2, 3]);
    let b: BigUint<4> = BigUint::from_le_bytes(&[7, 2, 3]);

    let (q, r) = a.div(b);

    assert(q.eq(BigUint::one()));
    assert(r.eq(BigUint::zero()));
}

fn test_div3() {
    // (256*256 + 2)/256 = (256, 2)
    let a: BigUint<4> = BigUint::from_le_bytes(&[2, 0, 1]);
    let b: BigUint<4> = BigUint::from_le_bytes(&[0, 1]);

    let (q, r) = a.div(b);
    let expected_q = BigUint::from_le_bytes(&[0, 1]);
    let expected_r = BigUint::from_le_bytes(&[2]);

    assert(q.eq(expected_q));
    assert(r.eq(expected_r));
}

fn test_div4() {
    let a: BigUint<1> = BigUint::from_le_bytes(&[7]);
    let b: BigUint<1> = BigUint::from_le_bytes(&[1]);

    let (q, r) = a.div(b);
    let expected_q = BigUint::from_le_bytes(&[7]);

    assert(q.eq(expected_q));
    assert(r.eq(BigUint::zero()));
}

fn test_div5() {
    // 7 + 256*2 + 256*256*3
    let a: BigUint<5> = BigUint::from_le_bytes(&[7, 2, 3]);
    let b: BigUint<5> = BigUint::from_le_bytes(&[0, 0, 0, 1]);

    let (q, r) = a.div(b);
    let expected_r: BigUint<5> = BigUint::from_le_bytes(&[7, 2, 3]);

    assert(q.eq(BigUint::zero()));
    assert(r.eq(expected_r));
}

fn test_sqrt1() {
    let a: BigUint<5> = BigUint::from_le_bytes(&[0, 0, 0, 0, 1]);
    let expected_sqrt: BigUint<5> = BigUint::from_le_bytes(&[0, 0, 1]);
    assert(a.sqrt().eq(expected_sqrt));
}

fn test_sqrt2() {
    let num: BigUint<8> = BigUint::from_le_bytes_32(
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 104, 203, 127, 80, 204, 48, 33, 247, 218, 52, 195, 241, 198]
    );
    let expected_sqrt: BigUint<8> = BigUint::from_le_bytes(&
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 136, 82, 99, 169, 29, 173, 225]
    );
    assert(num.sqrt().eq(expected_sqrt));
}

fn test_sqrt3() {
    // Not precise square
    let num: BigUint<8> = BigUint::from_le_bytes_32([
        84, 131, 32, 9, 152, 117, 67, 219,
        205, 41, 62, 115, 54, 121, 56, 101,
        148, 26, 32, 216, 170, 246, 39, 251,
        3, 35, 247, 218, 52, 195, 241, 198
    ]);

    let expected_sqrt: BigUint<8> = BigUint::from_le_bytes(&[
        231, 95, 119, 147, 127, 144, 147, 250,
        8, 137, 82, 99, 169, 29, 173, 225
    ]);
    assert(num.sqrt().eq(expected_sqrt));
}

fn test_sqrt4() {
    let a: BigUint<2> = BigUint::from_le_bytes(&[1, 0, 0, 0, 1]);
    let expected_sqrt: BigUint<2> = BigUint::from_le_bytes(&[0, 0, 1]);
    assert(a.sqrt().eq(expected_sqrt));
}

fn test_sqrt5() {
    let num: BigUint<12> = BigUint::from_le_bytes(&[
        54, 92, 89, 51, 110, 45, 220, 37, 12, 215, 31, 53,
        226, 99, 42, 136, 174, 218, 72, 5, 219, 84, 88, 79,
        47, 203, 166, 177, 29, 248, 176, 118, 211, 178, 213, 113,
        215, 200, 225, 141, 222, 206, 141, 245, 189, 52, 228, 67
    ]);

    let expected_sqrt: BigUint<12> = BigUint::from_le_bytes(&[
        234, 147, 133, 52, 134, 34, 197, 229, 40, 24, 71, 1,
        103, 112, 238, 171, 127, 97, 123, 39, 163, 129, 213, 131
    ]);
    assert(num.sqrt().eq(expected_sqrt));
}

fn test_sqrt6() {
    let num: BigUint<12> = BigUint::from_le_bytes(&[
        3, 114, 131, 27, 209, 209, 237, 216, 83, 208, 252, 34,
        139, 5, 35, 68, 33, 235, 97, 130, 162, 58, 84, 139,
        106, 241, 132, 3, 70, 86, 197, 169, 14, 78, 28, 47,
        135, 171, 48, 79, 117, 22, 71, 195, 179, 156, 149, 243
    ]);

    let expected_sqrt: BigUint<12> = BigUint::from_le_bytes(&[
        160, 93, 80, 149, 247, 156, 64, 180, 2, 132, 112, 226,
        74, 207, 234, 203, 104, 93, 0, 56, 75, 14, 183, 249
    ]);
    assert(num.sqrt().eq(expected_sqrt));
}
