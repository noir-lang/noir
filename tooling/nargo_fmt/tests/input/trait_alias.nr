trait Foo {
    fn foo(self) -> Self;
}

trait Baz = Foo;

impl Foo for Field {
    fn foo(self) -> Self { self }
}

fn baz<T>(x: T) -> T where T: Baz {
    x.foo()
}


pub trait Foo_2 {
    fn foo_2(self) -> Self;
}

pub trait Bar_2 {
    fn bar_2(self) -> Self;
}

pub trait Baz_2 = Foo_2 + Bar_2;

fn baz_2<T>(x: T) -> T where T: Baz_2 {
    x.foo_2().bar_2()
}

impl Foo_2 for Field {
    fn foo_2(self) -> Self {
        self + 1
    }
}

impl Bar_2 for Field {
    fn bar_2(self) -> Self {
        self + 2
    }
}


trait Foo_3 {
    fn foo_3(self) -> Self;
}

trait Bar_3<T> {
    fn bar_3(self) -> T;
}

trait Baz_3<T> = Foo_3 + Bar_3<T>;

fn baz_3<T, U>(x: T) -> U where T: Baz_3<U> {
    x.foo_3().bar_3()
}

impl Foo_3 for Field {
    fn foo_3(self) -> Self {
        self + 1
    }
}

impl Bar_3<bool> for Field {
    fn bar_3(self) -> bool {
        true
    }
}


fn main() {
    let x: Field = 0;
    let _ = baz(x);

    assert(0.foo_2().bar_2() == baz_2(0));

    assert(0.foo_3().bar_3() == baz_3(0));
}

