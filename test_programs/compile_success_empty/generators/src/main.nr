// TODO?
// the syntax for these return types is very difficult to get right :/
// for arguments this can be handled with a generic Env (or with Fn traits when we add them)
// but for return types neither fo these will help, you need to type out the exact type
fn make_counter() -> fn[(&mut Field,)]() -> Field {
    let mut x = &mut 0;

    || {
        *x = *x + 1;
        *x
    }
}

fn fibonacci_generator() -> fn[(&mut Field, &mut Field)]() -> Field {
    let mut x = &mut 1;
    let mut y = &mut 2;

    || {
        let old_x = *x;
        let old_y = *y;

        *y = *x + *y;
        *x = old_y;

        old_x
    }
}
// we'll be able to un-hardcode the array length if we have the ::<> syntax proposed in https://github.com/noir-lang/noir/issues/2458
fn get_some<Env>(generator: fn[Env]() -> Field) -> [Field; 5] {
    [0, 0, 0, 0, 0].map(|_| generator())
}

fn test_fib() {
    let fib = fibonacci_generator();

    assert(fib() == 1);
    assert(fib() == 2);
    assert(fib() == 3);
    assert(fib() == 5);

    assert(get_some(fib) == [8, 13, 21, 34, 55]);
}

fn test_counter() {
    let counter = make_counter();
    assert(counter() == 1);
    assert(counter() == 2);
    assert(counter() == 3);

    assert(get_some(counter) == [4, 5, 6, 7, 8]);
}

fn main() {
    test_fib();
    test_counter();
}
