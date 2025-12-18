unconstrained fn main() -> pub bool {
    foo(false)
}
unconstrained fn foo(mut a: bool) -> bool {
    if !a {
        true
    } else {
        let mut i = 0;
        loop {
            if i == 1 {
                break
            } else {
                i = i + 1;
                let b: [Field; 1] = {
                    a = (!a) < ((a as Field) == (a as Field));
                    assert(((a as Field) * (a as Field)) == if foo(false) { 1 } else { 1 });
                    println(a);
                    if foo(false) {
                        [(a as Field)]
                    } else {
                        [(a as Field)]
                    }
                };
                assert((a as Field) != b[0]);
            }
        }
        false
    }
}