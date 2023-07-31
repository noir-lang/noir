use dep::std;

fn f(x: &mut Field) -> Field {
    *x = *x - 1;
    1
}

fn g(x: &mut Field) -> Field {
    *x *= 2;
    1
}

fn h(x: &mut Field) -> Field {
    *x *= 3;
    1
}

fn selector(flag:&mut bool) -> fn(&mut Field) -> Field { //TODO: Can we have fn(&mut Field) -> () return type?
    let mut my_func = f;  

    if *flag {
        my_func = g;
    }
    else {
        my_func = h;
    };

    // Flip the flag for the next function call
    *flag = !(*flag);
    my_func
}

fn main() {
    
    let mut flag: bool = true;

    let mut x: Field = 100;
    let returned_func = selector(&mut flag);
    let _status = returned_func(&mut x);

    assert(x == 200);

    let mut y: Field = 100;
    let returned_func2 = selector(&mut flag);
    let _status2 = returned_func2(&mut y);

    assert(y == 300);

}
