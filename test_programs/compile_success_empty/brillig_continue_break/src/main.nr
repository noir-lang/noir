fn main() {
    // Safety: test program
    let bug = unsafe { foo() };
    assert(!bug);
}

unconstrained fn foo() -> bool {
    let mut i = 0;
    let mut bug = false;
    loop {
        if i == 3 {
            break;
            bug = true;
        } else if i == 2 {
            i += 1;
            continue;
            bug = true;
        }
        i += 1;
    }
    bug
}
