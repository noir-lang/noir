mod Foo {
    struct NewType{}
}


mod Bar {
    use crate::Foo::NewType as BarStruct;
    use crate::Foo::NewType;
} 

mod Baz {
    use crate::Bar::BarStruct;
    use crate::Bar::NewType;
}

// Baz is unable to find the Bar re-exports
// error: Could not resolve 'NewType' in path
// error: Could not resolve 'BarStruct' in path