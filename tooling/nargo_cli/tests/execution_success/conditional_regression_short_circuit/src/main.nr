use dep::std;

fn main(a: u32, mut c: [u32; 4], x: [u8; 5], result: pub [u8; 32]){
    //regression for short-circuit2
    if 35 == a {
        assert(false);
    }
    bar(a as Field);

    if a == 3 {
       c = test4();
    }
    assert(c[1] != 2);
    call_intrinsic(x, result);
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


fn call_intrinsic(x: [u8; 5], result: [u8; 32]) {
    let mut digest = std::hash::sha256(x);
    digest[0] = 5 as u8;
    digest = std::hash::sha256(x);
    assert(digest == result);
}

fn test4() -> [u32; 4] {
    let b: [u32; 4] = [1,2,3,4];
    b
}