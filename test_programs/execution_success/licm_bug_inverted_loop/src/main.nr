unconstrained fn main() {
    f(5);
}

#[inline_always]
unconstrained fn f(mut a: u8) {
    let mut i: u32 = 0;
    while (a != 0) {
        if (i == 2) {
            break
        }
        i += 1;
        a = if (a < 3) { 3 } else { 1 };
        println(a);
    }
}
