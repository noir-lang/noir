mod Foo {
    pub(crate) struct NewType {
        pub(crate) a: Field,
    }
}

mod Bar {
    pub(crate) use crate::Foo::NewType;
    pub(crate) use crate::Foo::NewType as BarStruct;
}

mod Baz {
    pub(crate) struct Works {
        pub(crate) a: Field,
    }
    pub(crate) use crate::Bar::BarStruct;
    pub(crate) use crate::Bar::NewType;
}

fn main(works: Baz::Works, fails: Baz::BarStruct, also_fails: Bar::NewType) -> pub Field {
    works.a + fails.a + also_fails.a
}
