use arbitrary::Unstructured;
use noirc_abi::Abi;
use noirc_frontend::monomorphization::ast::Program;

/// Generate an arbitrary monomorphized AST.
pub fn arb_program(_u: &mut Unstructured) -> arbitrary::Result<(Program, Abi)> {
    todo!()
}
