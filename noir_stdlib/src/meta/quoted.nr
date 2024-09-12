use crate::cmp::Eq;
use crate::option::Option;

impl Quoted {
    #[builtin(quoted_as_expr)]
    // docs:start:as_expr
    comptime fn as_expr(self) -> Option<Expr> {}
    // docs:end:as_expr

    #[builtin(quoted_as_module)]
    // docs:start:as_module
    comptime fn as_module(self) -> Option<Module> {}
    // docs:end:as_module

    #[builtin(quoted_as_trait_constraint)]
    // docs:start:as_trait_constraint
    comptime fn as_trait_constraint(self) -> TraitConstraint {}
    // docs:end:as_trait_constraint

    #[builtin(quoted_as_type)]
    // docs:start:as_type
    comptime fn as_type(self) -> Type {}
    // docs:end:as_type

    #[builtin(quoted_tokens)]
    // docs:start:tokens
    comptime fn tokens(self) -> [Quoted] {}
    // docs:end:tokens
}

impl Eq for Quoted {
    comptime fn eq(self, other: Quoted) -> bool {
        quoted_eq(self, other)
    }
}

#[builtin(quoted_eq)]
comptime fn quoted_eq(_first: Quoted, _second: Quoted) -> bool {}

