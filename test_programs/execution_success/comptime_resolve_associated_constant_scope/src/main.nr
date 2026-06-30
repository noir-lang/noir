// `Expr::resolve(Some(f))` resolves an expression in `f`'s module scope. That scope must
// also drive which trait is in scope for an associated-constant lookup, not the caller's
// module. Here `victim::Box<Field>` has an associated constant `N` from two different traits:
// `victim::LocalConst::N == 1` (in scope inside `victim`) and the caller's `CallerConst::N == 2`.
// Resolving `Box::<Field>::N` in `victim`'s scope must select `1`.
mod victim {
    pub struct Box<T> {
        pub value: T,
    }

    trait LocalConst {
        let N: u32;
    }

    impl LocalConst for Box<Field> {
        let N: u32 = 1;
    }

    pub fn scope_anchor() {}
}

trait CallerConst {
    let N: u32;
}

impl CallerConst for victim::Box<Field> {
    let N: u32 = 2;
}

fn main() {
    let selected = comptime {
        let victim_module = quote { victim }.as_module().unwrap();
        let mut anchor = Option::none();
        for f in victim_module.functions() {
            if f.name() == quote { scope_anchor } {
                anchor = Option::some(f);
            }
        }
        let expr = quote { Box::<Field>::N }.as_expr().unwrap().resolve(anchor);
        quote { $expr }
    };

    assert(selected == 1u32);
}
