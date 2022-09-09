use dep::std;

struct Foo {
    bar: Field,
}

struct Pair {
    first: Foo,
    second: Field,
}

impl Foo {
    fn default() -> Self {
        Self { bar: 0 }
    }
}

impl Pair {
    fn foo(p: Self) -> Foo {
        p.first
    }

    fn bar(self) -> Field {
        self.foo().bar
    }
}

fn main(x: Field, y: Field) {
    let first = Foo::default();
    let p = Pair { first, second: 1 };

    constrain p.bar() == x;
    constrain p.second == y;
}
    
