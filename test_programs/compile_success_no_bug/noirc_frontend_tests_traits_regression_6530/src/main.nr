
    pub trait From2<T> {
        fn from2(input: T) -> Self;
    }
    
    pub trait Into2<T> {
        fn into2(self) -> T;
    }
    
    impl<T, U> Into2<T> for U
    where
        T: From2<U>,
    {
        fn into2(self) -> T {
            T::from2(self)
        }
    }
    
    struct Foo {
        inner: Field,
    }
    
    impl Into2<Field> for Foo {
        fn into2(self) -> Field {
            self.inner
        }
    }
    
    fn main() {
        let foo = Foo { inner: 0 };
    
        // This works:
        let _: Field = Into2::<Field>::into2(foo);
    
        // This was failing with 'No matching impl':
        let _: Field = foo.into2();
    }
    