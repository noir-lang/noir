// A `#[fold]` function that fails *after* a nested `#[fold]` call has already returned.
//
// The call stack reported for the failure has to name `main` as the frame that entered `outer`;
// the successful `inner` call in between must not erase it.
fn main(x: Field) {
    assert(outer(x) == 0);
}

#[fold]
fn outer(x: Field) -> Field {
    let y = inner(x);
    assert(y == x, "outer failed after inner returned");
    y
}

#[fold]
fn inner(x: Field) -> Field {
    x + 1
}
