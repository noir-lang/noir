use dep::std;

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

    // Regression test for issue #547
    // Warning: it must be kept at the start of main
    let arr: [u8; 2] = [1, 2];
    if arr[0] != arr[1] {
        for i in 0..1 {
            constrain i != 2;
        }
    }

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

    //Regression for to_le_bits() constant evaluation 
    // binary array representation of u8 1
    let as_bits_hardcode_1 = [1, 0];
    let mut c1 = 0;
    for i in 0..2 {
        let mut as_bits = std::field::to_le_bits(arr[i] as Field, 2);
        c1 = c1 + as_bits[0] as Field;

        if i == 0 {
            constrain arr[i] == 1;// 1
            for k in 0..2 {
                constrain as_bits_hardcode_1[k] == as_bits[k];
            }
        }
        if i == 1 {
            constrain arr[i] == 2;//2
            for k in 0..2 {
                constrain as_bits_hardcode_1[k] != as_bits[k];
            }
        }
    }
    constrain c1==1;

    //Regression for Issue #579
    let result1_true = test(true);
    constrain result1_true.array_param[0] == 1;
    let result1_false = test(false);
    constrain result1_false.array_param[0] == 0;

    //Test case for short-circuit
    let mut data = [0 as u32; 32];
    let mut ba = a;
      for i in 0..32 {
        let i_u32 = i as u32;
        if i_u32 == a {
            for j in 0..4 {
                data[i + j] = c[4 - 1 - j];
                for k in 0..4 {
                    ba = ba +data[k];
                }
                if ba == 4864 {
                    c[3]=ba;
                }
            }
        }
    }
    constrain data[31] == 0;
    constrain ba != 13;
    //regression for short-circuit2
    if 35 == a {
        constrain false;
    }
    bar(a as Field);

    if a == 3 {
       c = test4();
    }
    constrain c[1] != 2;
    call_intrinsic(x, result);

    //Test case for conditional with arrays from function parameters
    let b = sort([1,2,3,4]);
    constrain b[0] == 1;

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
    if c[0] == 0 {
        x = 3;
    }
    constrain x == 2;

    //test2: loops!
    x = 0;
    x = a - a;
    for i in 0..4 {
        if c[i] == 0 {
            x = i as u32 +2;
        }
    }
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

    test5(4);

    // Regression for issue #661:
    let mut c_661 :[u32;1]=[0];
    if a > 5 {
        c_661 = issue_661_foo(issue_661_bar(c), a);
    } else {
        c_661 = issue_661_foo(issue_661_bar(c), x);
    }
    constrain c_661[0] < 20000;

    // Regression for predicate simplification
    safe_inverse(0);
}

fn test5(a : u32) {
    if a > 1 {
        let q = a / 2;
        constrain q == 2;
    }
}



fn foo() {
    let mut x = 1;
    x /= 0;
}

fn bar(x:Field) {
    if x == 15 {
        foo();
    }
}


struct MyStruct579 {
    array_param: [u32; 2]
}

impl MyStruct579 {
    fn new(array_param: [u32; 2]) -> MyStruct579 {
        MyStruct579 {
            array_param: array_param
        }
    }
}

fn test(flag: bool) -> MyStruct579 {
    let mut my_struct =  MyStruct579::new([0; 2]);

    if flag == true {
        my_struct=  MyStruct579::new([1; 2]);
    }
    my_struct
}

fn issue_661_foo(array: [u32;4], b:u32) ->[u32;1] {
    [array[0]+b]
}

fn issue_661_bar(a : [u32;4]) ->[u32;4] {
    let mut b:[u32;4] = [0;4];
    b[0]=a[0]+1;
    b
}


fn safe_inverse(n: Field) -> Field
{
    if n == 0 {
        0
    }
    else {
        1 / n
    }
}
