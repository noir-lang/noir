
        trait Foo {
            fn foo(self) -> Field;
        }

        trait Bar<T>: Foo {
            fn bar(self) -> Field {
                self.foo()
            }
        }

        struct MyStruct<T> {
            inner: Field,
        }

        trait MarkerTrait {}
        impl MarkerTrait for Field {}

        // `MyStruct<T>` implements `Foo` only when its generic type `T` implements `MarkerTrait`.
        impl<T> Foo for MyStruct<T>
        where
            T: MarkerTrait,
        {
            fn foo(self) -> Field {
                let _ = self;
                42
            }
        }

        // We expect this to succeed as `MyStruct<T>` satisfies `Bar`'s trait bounds
        // of implementing `Foo` when `T` implements `MarkerTrait`.
        impl<T> Bar<T> for MyStruct<T>
        where
            T: MarkerTrait,
        {
            fn bar(self) -> Field {
                31415
            }
        }

        fn main() {
            let foo: MyStruct<Field> = MyStruct { inner: 42 };
            let _ = foo.bar();
        }
    