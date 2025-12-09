fn main(c: bool) {
    let mut a: [&mut bool; 2] = [&mut false, &mut true];

    if (c) {
        a = [a[1], a[0]];
        a[0] = &mut true;
    };

    a = [&mut true, a[1]];
    a[1] = &mut false;
}
