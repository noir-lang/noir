// `c` is reassigned to a fresh array whose first element writes `c[0]` in place, while the
// loop-carried `a = c` would share `c`'s buffer if `c` were treated as killed. `a` must end up
// [1, 20]. Inputs come from `Prover.toml` so the result is not constant-folded before execution.
unconstrained fn main(a: [Field; 2], c: [Field; 2]) {
    let mut a = a;
    let mut c = c;
    let mut j: u32 = 0;
    while j < 2 {
        j += 1;
        c = [
            {
                let mut i: u32 = 0;
                while i < 1 {
                    i += 1;
                    c[0] = 99;
                }
                a[0]
            },
            20,
        ];
        a = c;
    }
    assert_eq(a, [1, 20]);
}
