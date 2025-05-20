trait Foo {
    type E: Bar;

    fn bar(self) -> Self::E;
}

trait Bar {
    fn bar_method(self);
}

impl Bar for i32 {
    fn bar_method(self) {}
}

impl Foo for i32 {
    type E = i32;

    fn bar(self) -> Self::E {
        1
    }
}

pub struct Baz<T> {
    t: T,
}

impl<T: Foo> Baz<T> {
    pub fn baz_method(t: T) {
        let bar = t.bar();
        bar.bar_method();
    }
}

impl<T: Foo> Eq for Baz<T> {
    fn eq(self, other: Self) -> bool {
        let bar = other.t.bar();
        bar.bar_method();
        true
    }
}

fn main() {}
