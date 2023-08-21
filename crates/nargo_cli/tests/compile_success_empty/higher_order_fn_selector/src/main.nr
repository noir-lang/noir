
fn g(x: &mut Field) -> () {
    *x *= 2;
}

fn h(x: &mut Field) -> () {
    *x *= 3;
}

fn selector(flag: &mut bool) -> fn(&mut Field) -> () {
    let my_func = if *flag {
        g
    } else {
        h
    };

    // Flip the flag for the next function call
    *flag = !(*flag);
    my_func
}

fn main() {
    
    let mut flag: bool = true;

    let mut x: Field = 100;
    let returned_func = selector(&mut flag);
    returned_func(&mut x);

    assert_eq(x, 200);

    let mut y: Field = 100;
    let returned_func2 = selector(&mut flag);
    returned_func2(&mut y);

    assert_eq(y, 300);

}
