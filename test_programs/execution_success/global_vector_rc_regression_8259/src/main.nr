global G_C: [bool] = #[true, false, true];
fn main(a: bool) {
    println(func_1(a, G_C));
    println(G_C);
}
fn func_1(a: bool, mut b: [bool]) -> [bool] {
    b[1] = a;
    b
}
