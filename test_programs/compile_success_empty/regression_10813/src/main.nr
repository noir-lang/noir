pub trait Foo {
    type Bar;
    let Baz: u32;

    fn foo(x: [Self::Bar; Self::Baz]) -> u32 {
        x.len()
    }
}

impl Foo for () {
    type Bar = ();
    let Baz: u32 = 0;
}

fn main() {
    assert(<() as Foo>::foo([(); 0]) == 0);
}
