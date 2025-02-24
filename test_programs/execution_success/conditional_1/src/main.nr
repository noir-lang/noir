fn sort(mut a: [u32; 4]) -> [u32; 4] {
    for i in 1..4 {
        for j in 0..i {
            if a[i] < a[j] {
                let c = a[j];
                a[j] = a[i];
                a[i] = c;
            }
        }
    }
    a
}

fn must_be_zero(x: u8) {
    assert(x == 0);
}

fn main(a: u32, mut c: [u32; 4], x: [u8; 5], result: pub [u8; 32]) {
    //Test case for short-circuit
    let mut data = [0 as u32; 32];
    let mut ba = a;
    for i in 0..32 {
        let i_u32 = i as u32;
        if i_u32 == a {
            for j in 0..4 {
                data[i + j] = c[4 - 1 - j];
                for k in 0..4 {
                    ba = ba + data[k];
                }
                if ba == 4864 {
                    c[3] = ba;
                }
            }
        }
    }
    assert(data[31] == 0);
    assert(ba != 13);
    //Test case for conditional with arrays from function parameters
    let b = sort([1, 2, 3, 4]);
    assert(b[0] == 1);

    if a == 0 {
        must_be_zero(0);
        c[0] = 3;
    } else {
        must_be_zero(1);
        c[0] = 1;
        c[1] = c[2] / a + 11 % a;
        let f1 = a as Field;
        assert(10 / f1 != 0);
    }
    assert(c[0] == 3);

    let mut y = 0;
    if a == 0 {
        let digest = std::hash::sha256(x);
        y = digest[0];
    } else {
        y = 5;
    }
    assert(y == result[0]);
    c = sort(c);
    assert(c[0] == 0);
    //test 1
    let mut x: u32 = 0;
    if a == 0 {
        c[0] = 12;
        if a != 0 {
            x = 6;
        } else {
            x = 2;
            assert(x == 2);
        }
    } else {
        x = 5;
        assert(x == 5);
    }
    if c[0] == 0 {
        x = 3;
    }
    assert(x == 2);
    //test2: loops
    let mut x: u32 = 0;
    x = a - a;
    for i in 0..4 {
        if c[i] == 0 {
            x = i as u32 + 2;
        }
    }
    assert(x == 0);
}
