// When `cond` is true, `h` is `q`, which points at `c`, so `*h = 7` overwrites `c = 5`.
// The unused `if` chains merge several alias classes with the class of `*p`.
unconstrained fn main(cond: bool) -> pub Field {
    let mut c: Field = 1;
    let mut q: &mut Field = &mut c;
    let mut p: &mut &mut Field = &mut q;
    let arr = [(&mut p, &mut q)];

    let mut s1: Field = 0;
    let mut s2: Field = 0;
    let mut s3: Field = 0;
    let mut s4: Field = 0;
    let mut s5: Field = 0;
    let mut s6: Field = 0;
    let mut s7: Field = 0;
    let mut s8: Field = 0;
    let t1 = if cond { &mut s1 } else { &mut s2 };
    let t2 = if cond { &mut s3 } else { &mut s4 };
    let t3 = if cond { &mut s5 } else { &mut s6 };
    let t4 = if cond { &mut s7 } else { &mut s8 };
    let t5 = if cond { t1 } else { t2 };
    let t6 = if cond { t3 } else { t4 };
    let t7 = if cond { t5 } else { t6 };

    let mut r1: &mut Field = &mut s1;
    let mut r2: &mut Field = &mut s1;
    let r3 = if cond { &mut r1 } else { &mut r2 };
    let z = if cond { &mut q } else { &mut r1 };

    let h: &mut Field = if cond { q } else { *z };
    c = 5;
    *h = 7;
    let _ = (arr, r3, t7, p);
    c
}
