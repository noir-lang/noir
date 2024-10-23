mod Foo {
    struct NewType {
        a: Field,
    }
}

mod Bar {
    use crate::Foo::{NewType, NewType as BarStruct};
}

mod Baz {
    struct Works {
        a: Field,
    }
    use crate::Bar::{BarStruct, NewType};
}

fn main(works: Baz::Works, fails: Baz::BarStruct, also_fails: Bar::NewType) -> pub Field {
    works.a + fails.a + also_fails.a
}
