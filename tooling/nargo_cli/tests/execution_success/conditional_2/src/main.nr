use dep::std;


fn must_be_zero(x: u8) {
    assert(x == 0);
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

fn main(a: u32, mut c: [u32; 4]) {
  
    test3(1);

    if a == 0 {
        c = test4();
    } else {
        assert(c[1] != 2);
    }
    if false {
        c[1] = 5;
    }
    assert(c[1] == 2);

    test5(4);
    
    // Test case for function synchronisation
    let mut c_sync = 0;
    if a == 42 {
        c_sync = foo2();
    } else {
        c_sync = foo2() + foo2();
    }
    assert(c_sync == 6);
}

fn test5(a : u32) {
    if a > 1 {
        let q = a / 2;
        assert(q == 2);
    }
}

fn foo2() -> Field {
    3
}
