fn main() -> pub [[Field; 1]; 1] {
    let a = unsafe { foo([[0xcafebabe]]) };
    assert(a[0][0] == 0xcafebabe);
    a
}

unconstrained fn foo(mut a: [[Field; 1]; 1]) -> [[Field; 1]; 1] {
    let mut b = bar(a)[0];

    let mut x = 0;
    while (x != 0) {}

    b[0] = 0xdeadbeef;
    a
}

fn bar(mut a: [[Field; 1]; 1]) -> [[Field; 1]; 1] {
    a
}
