fn main() {
    let _ = Foo { func: foo };
}

fn foo() {}

struct Foo {
    func: unconstrained fn() -> (),
}
