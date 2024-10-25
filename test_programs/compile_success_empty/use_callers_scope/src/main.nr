#[bar::struct_attr]
pub struct Foo {}

pub struct Bar {}

#[bar::fn_attr]
fn main() {}

mod bar {
    #[use_callers_scope]
    pub comptime fn struct_attr(_: StructDefinition) {
        let _ = quote { Bar }.as_type();
    }

    #[use_callers_scope]
    pub comptime fn fn_attr(_: FunctionDefinition) {
        let _ = quote { Bar }.as_type();
        let _ = nested();

        // Ensure closures can still access Bar even
        // though `map` separates them from `fn_attr`.
        let _ = &[1, 2, 3].map(|_| quote { Bar }.as_type());
    }

    // use_callers_scope should also work nested
    #[use_callers_scope]
    comptime fn nested() -> Type {
        quote { Bar }.as_type()
    }
}
