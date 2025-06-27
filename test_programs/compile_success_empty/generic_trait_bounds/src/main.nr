trait Trait {
    fn foo(self);
}

pub fn foo<T: Trait>(x: T) {
    x.foo();
}

pub struct Foo {}

impl Foo {
    fn foo<T: Trait>(x: T) {
        x.foo();
    }
}

trait Trait2 {
    fn foo<T: Trait>(self, x: T) {
        x.foo();
    }
}

impl Trait2 for Foo {
    fn foo<T: Trait>(self, x: T) {
        x.foo();
    }
}

pub struct Bar<T> {}

impl<T: Trait> Bar<T> {
    fn bar(x: T) {
        x.foo();
    }

    fn baz(self) {}
}

impl<U: Trait> Trait2 for Bar<U> {
    fn foo<T: Trait>(self, x: T) {
        self.baz();
        x.foo();
    }
}

fn main() {}
