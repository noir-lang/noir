struct Pair {
    x: i32,
    y: i32,
}

fn main() {
    let mut p = Pair { x: 1, y: 2 };
    let get_x = || p.x;
    p.x = 99;
    let captured_x = get_x();
    assert_eq(captured_x, 1);
}
