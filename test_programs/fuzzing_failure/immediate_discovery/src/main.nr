#[fuzz]
fn fuzz(a:Field, b:Field) {
    assert(a!=b);
}
