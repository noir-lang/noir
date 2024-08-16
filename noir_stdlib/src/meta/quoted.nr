use crate::cmp::Eq;
use crate::option::Option;

impl Quoted {
    #[builtin(quoted_as_expr)]
    fn as_expr(self) -> Option<Expr> {}

    #[builtin(quoted_as_module)]
    fn as_module(self) -> Option<Module> {}

    #[builtin(quoted_as_trait_constraint)]
    fn as_trait_constraint(self) -> TraitConstraint {}

    #[builtin(quoted_as_type)]
    fn as_type(self) -> Type {}
}

impl Eq for Quoted {
    fn eq(self, other: Quoted) -> bool {
        quoted_eq(self, other)
    }
}

#[builtin(quoted_eq)]
fn quoted_eq(_first: Quoted, _second: Quoted) -> bool {}

