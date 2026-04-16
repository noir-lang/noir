unconstrained fn main() -> pub Field {
    let mut value = 1;
    foo(&mut value, &value)
}

unconstrained fn foo(v0: &mut Field, v1: &Field) -> Field {
    let _ = *v1;
    *v0 = 2;
    *v1
}
