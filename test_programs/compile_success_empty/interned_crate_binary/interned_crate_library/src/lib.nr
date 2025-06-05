#[derive_via(derive_foo)]
pub trait Foo {
    fn foo(self);
}

comptime fn derive_foo(t: TypeDefinition) -> Quoted {
    let typ = t.name();
    quote {
        impl $crate::Foo for $typ {
            fn foo(self) {
                $crate::my_public_fn_that_wont_be_imported();
            }
        }
    }
}

// Ensure $crate works in the same crate it is defined in
#[derive(Foo)]
struct LibStruct {}

pub fn lib_fn() {
    LibStruct {}.foo();
}

pub fn my_public_fn_that_wont_be_imported() {}
