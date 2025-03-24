struct Foo<let F: Field> {}

impl<let F: Field> Foo<F> {
    fn size(self) -> Field {
        let _ = self;
        F
    }
}

// 2^32 - 1
global A: Field = 4294967295;

// Avoiding overflow succeeds:
// fn foo<let A: Field>() -> Foo<A> {
fn foo<let A: Field>() -> Foo<A + A> {
    Foo {}
}

fn main() {
    let _ = foo::<A>().size();
}
