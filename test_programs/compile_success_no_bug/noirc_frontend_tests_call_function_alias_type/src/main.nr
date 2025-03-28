type Alias<Env> = fn[Env](Field) -> Field;

fn main() {
    call_fn(|x| x + 1);
}

fn call_fn<Env>(f: Alias<Env>) {
    assert_eq(f(0), 1);
}
