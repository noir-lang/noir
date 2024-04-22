fn g(x: &mut field) -> () {
    *x *= 2;
}

fn h(x: &mut field) -> () {
    *x *= 3;
}

fn selector(flag: &mut bool) -> fn(&mut field) -> () {
    let my_func = if *flag { g } else { h };
    // Flip the flag for the next function call
    *flag = !(*flag);
    my_func
}

fn main() {
    let mut flag: bool = true;

    let mut x: field = 100;
    let returned_func = selector(&mut flag);
    returned_func(&mut x);

    assert(x == 200);

    let mut y: field = 100;
    let returned_func2 = selector(&mut flag);
    returned_func2(&mut y);

    assert(y == 300);
}
