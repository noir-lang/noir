fn matrix_mul_2(a: [4]u32, b:[4]u32) ->[4]u32{
    let mut c:[4]u32 = [0 as u32; 4];
    for i in 0..2 {
        for j in 0..2 {
            c[i+2*j] = 0 as u32;
            for k in 0..2 {
               c[i+2*j] = c[i+2*j] + a[i+2*k]*b[k+2*j];
            };
        };
    };
    c
}

fn matrix_mul_10(a: [100]u32, b:[100]u32) ->[100]u32{
    let mut c:[100]u32 = [0 as u32; 100];
    for i in 0..10 {
        for j in 0..10 {
            c[i+10*j] = 0 as u32;

            for k in 0..10 {
               c[i+10*j] = c[i+10*j] + a[i+10*k]*b[k+10*j];
            };
        };
    };
    c
}


fn siggy(x: u32) -> u32 {
    x * (10 as u32)
}


fn test4 (mut a: [4]u32) -> [4]u32 {
    for i in 3..4 {
    a[i] = siggy(a[i-2]);
     };
    a
}

fn main(a: [100]u32,b: [100]u32, c: [4]u32, d: [4]u32){  
    let e = matrix_mul_10(a,b);
    constrain e[6]==1866842232;
    let f = matrix_mul_2(c,d);
    constrain f[3]==2082554100;

    let mut a:[4]u32 = [1 as u32, 2 as u32, 3 as u32, 4 as u32];
    a = test4(a);
    constrain a[3] == 20;
    a =test4(c);
    constrain a[3] == c[1]*10;
}
