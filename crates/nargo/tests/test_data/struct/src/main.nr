use dep::std;

struct Foo {
    bar: Field,
    array: [Field; 2],
}

struct Pair {
    first: Foo,
    second: Field,
}

impl Foo {
    fn default(x: Field,y: Field) -> Self {
        Self { bar: 0, array: [x,y] }
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
    let first = Foo::default(x,y);
    let p = Pair { first, second: 1 };

    constrain p.bar() == x;
    constrain p.second == y;
    constrain p.first.array[0] != p.first.array[1];
}
    
