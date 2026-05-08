global G_A: Option<bool> = Option::none();
fn main() {
    let a: Option<bool> = Option::none();
    f(a);
    f(G_A);
}
fn f(_arg: Option<bool>) {}
