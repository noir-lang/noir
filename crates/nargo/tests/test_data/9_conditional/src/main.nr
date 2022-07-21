


fn main(a: u32, mut c: [4]u32
    ){  
    //test 1
    let mut x: u32 = 0;
    if a == 0 {
        c[1]=12;
        if a!=0 {
            x=6;
        }
        else {
            x=2;
            constrain x == 2;
        }
    } else {
        x = 5;
        constrain x == 5;
    }
    constrain c[1] == 12;
    constrain x == 2;

    //test2: loops!
    x = 0;
    x= a-a;
    for i in 0..3 {
        if c[i] == 0
        {
            x=i as u32 +2;
        }
    };
    constrain x==0;
}
