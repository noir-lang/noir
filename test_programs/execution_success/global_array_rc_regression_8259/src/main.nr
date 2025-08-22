global G_C: [bool; 3] = [true, false, true];
fn main(a: bool) -> pub [bool; 3] {
    let b = func_1(a, G_C);
    if a {
        G_C
    } else {
        b
    }
}
fn func_1(a: bool, mut b: [bool; 3]) -> [bool; 3] {
    b[1] = a;
    b
}
