use dep::std;

struct Foo {
    bar: Field,
}

struct Pair {
    first: Foo,
    second: Field,
}

impl Pair {
    fn foo(p: Pair) -> Foo {
        p.first
    }

    fn bar(p: Pair) -> Field {
        p.foo().bar
    }
}

fn main(x : Field, y : Field) {
    let first = Foo { bar: 0 };
    let p = Pair { first, second: 1 };

    constrain p.bar() == x;
    constrain p.second == y;
}
    
