// A call that receives an immutable reference can still read through it, so the
// store before `reader(v1)` is observable and must not be eliminated as dead by
// the later store. If load/store forwarding wrongly drops the first store,
// `reader` reads the initial 0 and the assertion incorrectly passes.
// Regression test for https://github.com/noir-lang/noir-claude/issues/1378.
fn main() {
    let v0 = &mut 0;
    foo(v0, v0);
}

fn foo(v0: &mut Field, v1: &Field) {
    *v0 = 1;
    reader(v1);
    *v0 = 2;
}

fn reader(v0: &Field) {
    let v1 = *v0;
    assert_eq(v1, 0);
}
