mod Foo {
    struct NewType{
        a: field,
    }
}

mod Bar {
    use crate::Foo::NewType as BarStruct;
    use crate::Foo::NewType;
}

mod Baz {
    struct Works {
        a: field,
    }
    use crate::Bar::BarStruct;
    use crate::Bar::NewType;
}

fn main(works: Baz::Works, fails: Baz::BarStruct, also_fails: Bar::NewType) -> pub field {
    works.a + fails.a + also_fails.a
}
