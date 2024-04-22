use crate::{macros_api::NodeInterner, node_interner::FuncId};

mod errors;
mod hir_to_ast;
mod interpreter;
mod scan;
mod tests;
mod value;

pub use interpreter::Interpreter;

/// Scan through a function, evaluating any CompTime nodes found.
/// These nodes will be modified in place, replaced with the
/// result of their evaluation.
pub fn scan_function(function: FuncId, interner: &mut NodeInterner) {
    let mut interpreter = Interpreter::new(interner);
    interpreter.scan_function(function);
}
