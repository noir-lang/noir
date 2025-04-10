use super::visitor::ExpressionVisitor;

/// Check if the AST has a `Call` in it.
#[derive(Default)]
pub(crate) struct HasCall(pub bool);

impl ExpressionVisitor for HasCall {
    fn should_continue(&self) -> bool {
        !self.0
    }
    fn visit_call(&mut self, _: &mut noirc_frontend::monomorphization::ast::Call) -> bool {
        self.0 = true;
        false
    }
}
