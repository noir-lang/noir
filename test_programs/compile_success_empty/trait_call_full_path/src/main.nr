mod foo {
    trait Trait {
        fn me(self) -> Self;
    }

    impl Trait for Field {
        fn me(self) -> Self {
            self
        }
    }
}

fn main(x: Field) {
    let _ = foo::Trait::me(x);
}
