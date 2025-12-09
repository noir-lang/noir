unconstrained fn main(v2: u32) {
    let v0 = &mut 0;
    let v7 = foo(v0, v0, v2);
    assert_eq(v7, 100);
}

unconstrained fn foo(v0: &mut Field, v1: &mut Field, v2: u32) -> Field {
    let mut v3 = [v0];
    v3[v2] = v1;
    let _v5 = *v0;
    let v6 = v3[v2];
    *v6 = 100;
    *v0
}
