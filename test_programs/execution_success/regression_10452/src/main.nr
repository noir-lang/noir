fn main() {
    test();

    comptime {
        test();
    }
}

fn test() {
    let x = &mut 0;
    let inc = || {
        *x += 1;
        *x
    };

    let _ = [inc(); 0];
    assert_eq(*x, 1);

    let _ = [inc(); 100];
    assert_eq(*x, 2);
}
