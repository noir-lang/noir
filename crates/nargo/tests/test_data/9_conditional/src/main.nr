use dep::std;

fn sort(mut a: [u32; 4]) -> [u32; 4] { 
    for i in 0..3 {
        for j in 0..3-i {
            if(a[j] > a[j+1]) {
                let c = a[j];
                a[j] = a[j+1];
                a[j+1]= c;
            }
        };
    };
    a
}

fn call_intrinsic(x: [u8; 5], result: [u8; 32]) {
    let mut digest = std::hash::sha256(x);
    digest[0] = 5 as u8;
    digest = std::hash::sha256(x);
    constrain digest == result;
}

fn must_be_zero(x: u8) {
    constrain x == 0;
}

fn test3 (x: u8) {
    if x == 0 {
        must_be_zero(x);
    }
}

fn test4() -> [u32; 4] {
    let b: [u32; 4] = [1,2,3,4];
    b
}

fn main(a: u32, mut c: [u32; 4], x: [u8; 5], result: pub [u8; 32]){
    //Issue reported in #421
    if a == c[0] {
        constrain c[0] == 0;
    } else  {
        if a == c[1] {
            constrain c[1] == 0;
        } else  {
            if a == c[2] {
                constrain c[2] == 0;
            }
        }
    }

    //Regression for to_bits() constant evaluation 
    let arr : [u8; 2] = [ 1, 2 ];
    // binary array representation of u8 1
    let as_bits_hardcode_1 = [1, 0];
    let mut c1 = 0;
    for i in 0..2 {
        let mut as_bits = std::to_bits(arr[i] as Field, 2);
        c1 = c1 + as_bits[0] as Field;

        if i == 0 {
            constrain arr[i] == 1;// 1
            for k in 0..2 {
                constrain as_bits_hardcode_1[k] == as_bits[k];
            };
        }
        if i == 1 {
            constrain arr[i] == 2;//2
            for k in 0..2 {
                constrain as_bits_hardcode_1[k] != as_bits[k];
            };
        }
    };
    constrain c1==1;

    if a == 3 {
       c = test4();
    }
    constrain c[1] != 2;
    call_intrinsic(x, result);

    if a == 0 {
        must_be_zero(0);
        c[0] = 3;
    } else {
        must_be_zero(1);
        c[0] = 1;
        c[1] = c[2] / a + 11 % a;
        let f1 = a as Field;
        constrain 10/f1 != 0;
    }
    constrain c[0] == 3;

    let mut y = 0;
    if a == 0 {
        let digest = std::hash::sha256(x);
        y = digest[0];
    } else {
        y = 5;
    }
    constrain y == result[0];
    c = sort(c);
    constrain c[0]==0;

    //test 1
    let mut x: u32 = 0;
    if a == 0 {
        c[0] = 12;
        if a != 0 {
            x = 6;
        } else {
            x = 2;
            constrain x == 2;
        }
    } else {
        x = 5;
        constrain x == 5;
    }
    constrain x == 2;

    //test2: loops!
    x = 0;
    x = a - a;
    for i in 0..4 {
        if c[i] == 0 {
            x = i as u32 +2;
        }
    };
    constrain x == 0;

    test3(1);

    if a == 0 {
        c = test4();
    } else {
        constrain c[1] != 2;
    }
    if false {
        c[1] = 5;
    }
    constrain c[1] == 2;
}

