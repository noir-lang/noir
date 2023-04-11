fn matrix_mul_2(a: [u32; 4], b: [u32; 4]) ->[u32; 4] {
    let mut c = [0 as u32; 4];
    for i in 0..2 {
        for j in 0..2 {
            c[i+2*j] = 0;
            for k in 0..2 {
               c[i+2*j] += a[i+2*k] * b[k+2*j];
            }
        }
    }
    c
}

fn matrix_mul_10(a: [u32; 100], b: [u32; 100]) -> [u32; 100] {
    let mut c = [0 as u32; 100];
    for i in 0..10 {
        for j in 0..10 {
            c[i+10*j] = 0 as u32;

            for k in 0..10 {
               c[i+10*j] += a[i+10*k] * b[k+10*j];
            }
        }
    }
    c
}


fn siggy(x: u32) -> u32 {
    x * (10 as u32)
}


fn test4 (mut a: [u32; 4]) -> [u32; 4] {
    for i in 3..4 {
        a[i] = siggy(a[i-2]);
    }
    a
}

fn iterate1(mut a0: u32) -> u32{
    let mut t1 = 0 as u32;
    let mut t2 = 0 as u32;
    let mut a = 1 as u32;
    let mut f = 2 as u32;
    let mut g = 3 as u32;
    let mut h = 4 as u32;

    for _i in 0..2 {
        t1 = h;
        h = g;
        g = f;
        a = t1 + t2;
    }
    a0 += a;
    a0
}

fn array_noteq(a: [u32; 4], b: [u32; 4]) {
    constrain a != b;
}

fn test3(mut b: [Field; 4]) -> [Field; 4] {
    for i in 0..4 {
        b[i] = i;
    }
    b
}

fn iterate2(mut hash: [u32; 8]) -> [u32; 8] {
    let mut t1 = 0 as u32;
  
    let mut a = hash[0];
    let mut e = hash[4];
    let mut f = hash[5];
    let mut g = hash[6];
    let mut h = hash[7];

    for _i in 0..2 { 
        t1 = ch2(e, f);
        h = g;
        g = f;
        a = t1;
    }

    hash[0] = hash[0] + a;
    hash
}

fn iterate3( mut hash: [u32; 8]) -> [u32; 8] {
    let mut t1 = 0 as u32;
    let mut t2 = 0 as u32;
    let mut a = hash[0];
    let mut b = hash[1];
    let mut c = hash[2];
    let mut d = hash[3];
    let mut e = hash[4];
    let mut f = hash[5];
    let mut g = hash[6];
    let mut h = hash[7];

    for _i in 0..3 { 
        t1 = ep2(e)+ch2(e, f);
        h = g;
        g = f;
        a = t1+t2;
    }
    constrain a == 2470696267;
    hash[0] = hash[0] + a;
    hash[1] = hash[1] + b;
    hash[2] = hash[2] + c;
    hash[3] = hash[3] + d;
    hash[4] = hash[4] + e;
    hash[5] = hash[5] + f;
    hash[6] = hash[6] + g;
    hash[7] = hash[7] + h;
    hash
}


fn test5() {
    let mut sha_hash = [
        0 as u32, 1, 2, 3,
        4, 5, 6, 7
    ];

    sha_hash = iterate2(sha_hash);

    constrain sha_hash[0] == 9;
}


fn ch2(x: u32, y: u32) -> u32 {
    x + y
}

fn ep2(x: u32) -> u32 {
    (2 as u32) * too(x)
}

fn too(x: u32) -> u32 {
    (x + 17 as u32) * (x + 3 as u32)
}

fn test6(x: [u8; 32]) ->  [u32; 8] {
    let mut sha_m = [0 as u32; 64];

    let mut sha_hash = [
        1 as u32, 2, 3, 4, 5, 6, 7, 8
    ];

    let mut buffer = [0 as u8; 64];
    for i in 0..32 {  
       buffer[i] = x[i];
    }

    sha_m = iterate6_1(sha_m, buffer);
    sha_hash = iterate6_2(sha_m, sha_hash);
    sha_hash
}

fn iterate6_1(mut sha_m: [u32; 64], next_chunk: [u8; 64]) -> [u32; 64] {
    let mut j = 0;
    for i in 0..16 {
        j = (i ) * 4;
        sha_m[i] = ((next_chunk[j] as u32) << 24 as u32)  
            | ((next_chunk[j + 1] as u32) << 16 as u32) 
            | ((next_chunk[j + 2] as u32) << 8 as u32) 
            | (next_chunk[j + 3] as u32);
    }
    for i in 16..64 {
        sha_m[i] = sig1(sha_m[i - 2])+(sha_m[i - 7])+(sig0(sha_m[i - 15]))+(sha_m[i - 16]);
    }
    sha_m
}

fn iterate6_2(sha_m: [u32; 64], mut hash: [u32; 8]) -> [u32; 8] {
    let mut t1 = 0 as u32;
    let mut t2 = 0 as u32;
    let mut a = 1 as u32;
    let mut b = 2 as u32;
    let mut c = 3 as u32;
    let mut d = 4 as u32;
    let mut e = 5 as u32;
    let mut f = 6 as u32;
    let mut g = 7 as u32;
    let mut h = 8 as u32;

    for i in 0..11 { 
        t1 = h + ep1(e) + ch(e, f, g) + sha_m[i];
        t2 = epo(a) + maj(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d+t1;
        d = c;
        c = b;
        b = a;
        a = t1+t2;
    }

    hash[0] = hash[0]+a;
    hash[1] = hash[1]+b;
    hash[2] = hash[2]+c;
    hash[3] = hash[3]+d;
    hash[4] = hash[4]+e;
    hash[5] = hash[5]+f;
    hash[6] = hash[6]+g;
    hash[7] = hash[7]+h;
    hash
}

fn rot_right(a: u32, b: u32) -> u32 {
    ((a >> b) | (a << (32 as u32 - b)))
}


fn ch(x: u32, y: u32, z: u32) -> u32 {
    ((x & y) ^ (!x & z))
}


fn maj(x: u32, y: u32, z: u32) -> u32 {
    ((x & y) ^ (x & z) ^ (y & z))
}


fn epo(x: u32) -> u32 {
    (rot_right(x, 2) ^ rot_right(x, 13) ^ rot_right(x, 22))
}

fn ep1(x: u32) -> u32 {
   (rot_right(x, 6) ^ rot_right(x, 11) ^ rot_right(x, 25))
}

fn sig0(x: u32) -> u32 {
    (rot_right(x, 7) ^ rot_right(x, 18) ^ (x >> 3))
}

fn sig1(x: u32) -> u32 {
    (rot_right(x, 17) ^ rot_right(x, 19) ^ (x >> 10))
}


fn main(a: [u32; 100], b: [u32; 100], c: [u32; 4], mut d: [u32; 4], m: [u8; 32]) {  
    let e = matrix_mul_10(a,b);
    constrain e[6] == 1866842232;
    let f = matrix_mul_2(c,d);
    constrain f[3] == 2082554100;

    let mut a = [1 as u32, 2, 3, 4];
    a = test4(a);
    constrain a[3] == 20;
    a = test4(c);
    constrain a[3] == c[1] * 10;

    d[0] += c[0];
    d[0] += c[1];
    constrain d[0] == 2739986880;

    let h = iterate1(1);
    constrain h == 4;

    let x = d;
    array_noteq(x, [d[0], d[1], d[2], 0]);

    let mut h5 = [d[0] as Field, d[1] as Field, d[2] as Field, d[3] as Field];
    let t5 = test3(h5);
    constrain t5[3] == 3;
    h5 = test3(h5);
    constrain h5[3] == 3;

    test5();

    let mut sha_hash = [
        0x6a09e667 as u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
    ];
    sha_hash = iterate3(sha_hash);

    let h6 = test6(m);
    constrain h6[0]== 523008072; //31.. 3800709683;
}
