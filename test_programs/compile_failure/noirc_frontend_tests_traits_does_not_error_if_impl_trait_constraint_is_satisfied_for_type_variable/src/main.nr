
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T> where T: Greeter {
            fn greet(object: T) {
                object.greet();
            }
        }

        pub struct Bar;

        impl<T> Foo<T> for Bar where T: Greeter {
        }

        fn main() {
        }
    