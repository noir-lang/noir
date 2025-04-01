
    mod foo {
        pub struct Foo {
            inner: Field,
        }
    
        impl Foo {
            pub fn new(inner: Field) -> Self {
                Self { inner }
            }
        }
    }
    
    use foo::Foo;
    
    fn main() {
        let _ = Foo::new(5);
        let _ = comptime { Foo::new(5) };
    }
    