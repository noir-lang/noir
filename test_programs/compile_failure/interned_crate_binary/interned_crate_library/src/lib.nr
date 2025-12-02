#[derive_via(derive_foo)]
pub trait Foo {
    fn foo(self);
}

comptime fn derive_foo(t: TypeDefinition) -> Quoted {
    let typ = t.name();
    quote {
        impl $crate::Foo for $typ {
            fn foo(self) {
                $crate::my_private_fn_that_cannot_be_imported();
            }
        }
    }
}

// Accessing the private method via $crate works in the crate it was defined in.
#[derive(Foo)]
struct LibStruct {}

pub fn lib_fn() {
    LibStruct {}.foo();
}

fn my_private_fn_that_cannot_be_imported() {}
