// A `pub(crate)` function in a private dependency submodule must not become callable
// through `FunctionDefinition::as_typed_expr`, even when it is reached by enumerating the
// dependency's modules and functions via the comptime meta API.
comptime fn bypass(_m: Module) -> Quoted {
    let lib_root = quote { victim_lib }.as_module().unwrap();

    let mut internal_mod = Option::none();
    for child in lib_root.child_modules() {
        if child.name() == quote { internal } {
            internal_mod = Option::some(child);
        }
    }

    let mut raw_compute = Option::none();
    for f in internal_mod.unwrap().functions() {
        if f.name() == quote { raw_compute } {
            raw_compute = Option::some(f);
        }
    }

    let raw = raw_compute.unwrap().as_typed_expr();
    quote {
        pub fn smuggled(x: Field) -> Field {
            $raw(x)
        }
    }
}

#[bypass]
mod smuggle {}

fn main() {
    assert(smuggle::smuggled(999) == 999000);
}
