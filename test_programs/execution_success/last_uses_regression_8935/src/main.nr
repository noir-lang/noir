unconstrained fn main(a: [u16; 1], b: [u16; 1]) {
    case1(a);
    case2(b);
}

unconstrained fn case1(array: [u16; 1]) {
    if false {} else {
        let mut d = array;
        if (true) {
            d[0] = 2;
            assert(array[0] != 2);

            // This println is needed to ensure d is not optimized out
            println(d);
        } else {
            ()
        }
    }
}

unconstrained fn case2(array: [u16; 1]) {
    if true {
        if true {
            let mut foo = array;
            foo[0] = 0;

            // This println is needed to ensure foo is not optimized out
            println(foo);
        }
    }

    if true {
        if false {
            ()
        } else {
            assert(array[0] != 0);
        }
    }
}
