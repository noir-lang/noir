global G_A: Option<u1> = Option::none();
fn main() {
    let a: Option<u1> = Option::none();
    f(a);
    f(G_A);
}
fn f(_arg: Option<u1>) {}
