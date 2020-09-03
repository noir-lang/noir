use libnoirc_ast::{BlockStatement, Ident, Type};

#[derive(Clone, Debug)]
pub struct Function {
    pub parameters: Vec<(Ident, Type)>,
    pub body: BlockStatement,
    // pub env: Environment, // Should we allow closures? XXX: Adding an environment means that order of function declaration will matter.
}

// Should we have functions as first order functions, and what about closures?
// First order functions may not be necessary. But closures can be useful.
// For closures, we need another struct with an environment
