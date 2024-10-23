mod foo {
    pub trait Trait {
        fn me(self) -> Self;
    }

    impl Trait for Field {
        fn me(self) -> Self {
            self
        }
    }
}

use foo::Trait::{me, self};

fn main(x: Field) {
    let _ = foo::Trait::me(x);
    let _ = Trait::me(x);
    let _ = me(x);
}
