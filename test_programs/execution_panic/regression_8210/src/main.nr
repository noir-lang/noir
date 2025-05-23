// https://github.com/noir-lang/noir/issues/8210
fn main() {
    (42 as Field).bar();
}

trait Foo {
    let BAR: u32;

    fn bar(_: Self) -> u32 {
        Foo::BAR
    }
}

impl Foo for Field {
    let BAR: u32 = 254;
}
