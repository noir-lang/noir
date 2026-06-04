unconstrained fn main(idx: u32) -> pub Field {
    let v0 = &mut 0;
    let v1: [&mut Field; 2] = [&mut 0, &mut 0];
    foo(v0, idx, v1)
}

unconstrained fn foo(v0: &mut Field, v1: u32, mut v2: [&mut Field; 2]) -> Field {
    *v0 = 3;
    v2[v1] = v0;
    let v4 = v2[v1];
    bar(v4);
    *v0
}

unconstrained fn bar(v0: &mut Field) {
    *v0 = 1;
}
