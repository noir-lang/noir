// This program will fail immediately because the two starting inputs are the same
#[fuzz]
fn fuzz(a:Field, b:Field) {
    assert(a!=b);
}
