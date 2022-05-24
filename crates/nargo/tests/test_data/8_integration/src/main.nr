fn matrix_mul_2(a: [4]u32, b:[4]u32) ->[4]u32{
    let mut c:[4]u32 = [0 as u32, 0 as u32, 0 as u32, 0 as u32, ];
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
    let mut c:[100]u32 = [0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 
    0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, 0 as u32, ];
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


fn main(a: [100]u32,b: [100]u32, c: [4]u32, d: [4]u32){  
    let e = matrix_mul_10(a,b);
    constrain e[6]==1866842232;
    let f = matrix_mul_2(c,d);
    constrain f[3]==2082554100;
}