global G_A: [(bool, bool, str<4>); 3] =
    [(false, true, "LZUT"), (false, true, "MFFA"), (false, true, "XPHT")];

fn main() -> pub Field {
    let mut a: ((str<4>, Field, str<4>, str<3>, bool), [&mut u8; 2]) = (
        (G_A[2].2, (unsafe { func_4() } as Field), G_A[1].2, "TWC", false), [(&mut 57), (&mut 219)],
    );
    for idx_d in 0..1 {
        // a.0 = a.0;
    }
    a.0 = (a.0.2, 0, a.0.0, a.0.3, ((128 % (*a.1[0])) == (*a.1[1])));
    0
}

unconstrained fn func_4() -> bool {
    let mut a = G_A[1].2;
    println(a);
    true
}
