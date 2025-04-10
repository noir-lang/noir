use noirc_frontend::monomorphization::ast::{
    Assign, Binary, Call, Cast, Expression, For, Ident, If, Index, LValue, Let, Literal, Match,
    Unary, While,
};

/// Visitor for the contents of [Expression] representing the AST.
///
/// Gets mutable references so it can manipulate the expressions if needed.
///
/// Where a `bool` is returned it indicates whether we want to visit the children
/// of the visited expression.
pub(crate) trait ExpressionVisitor {
    /// By default every `visit_*` will call this method to decide whether to
    /// continue visiting the AST or stop. This is to facilitate early returns
    /// if a visitor has concluded its business.
    fn should_continue(&self) -> bool {
        true
    }
    fn visit_ident(&mut self, _: &mut Ident) {}
    fn visit_literal(&mut self, _: &mut Literal) -> bool {
        self.should_continue()
    }
    fn visit_block(&mut self, _: &mut Vec<Expression>) -> bool {
        self.should_continue()
    }
    fn visit_unary(&mut self, _: &mut Unary) -> bool {
        self.should_continue()
    }
    fn visit_binary(&mut self, _: &mut Binary) -> bool {
        self.should_continue()
    }
    fn visit_index(&mut self, _: &mut Index) -> bool {
        self.should_continue()
    }
    fn visit_cast(&mut self, _: &mut Cast) -> bool {
        self.should_continue()
    }
    fn visit_for(&mut self, _: &mut For) -> bool {
        self.should_continue()
    }
    fn visit_loop(&mut self, _: &mut Expression) -> bool {
        self.should_continue()
    }
    fn visit_while(&mut self, _: &mut While) -> bool {
        self.should_continue()
    }
    fn visit_if(&mut self, _: &mut If) -> bool {
        self.should_continue()
    }
    fn visit_match(&mut self, _: &mut Match) -> bool {
        self.should_continue()
    }
    fn visit_tuple(&mut self, _: &mut Vec<Expression>) -> bool {
        self.should_continue()
    }
    fn visit_extract_tuple_field(&mut self, _: &mut Expression, _: &mut usize) -> bool {
        self.should_continue()
    }
    fn visit_call(&mut self, _: &mut Call) -> bool {
        self.should_continue()
    }
    fn visit_let(&mut self, _: &mut Let) -> bool {
        self.should_continue()
    }
    fn visit_constrain(&mut self, _: &mut Expression) -> bool {
        self.should_continue()
    }
    fn visit_assign(&mut self, _: &mut Assign) -> bool {
        self.should_continue()
    }
    fn visit_semi(&mut self, _: &mut Expression) -> bool {
        self.should_continue()
    }
    fn visit_clone(&mut self, _: &mut Expression) -> bool {
        self.should_continue()
    }
    fn visit_drop(&mut self, _: &mut Expression) -> bool {
        self.should_continue()
    }
    fn visit_break(&mut self) {}
    fn visit_continue(&mut self) {}
}

pub(crate) fn visit_expr<V: ExpressionVisitor>(visitor: &mut V, expr: &mut Expression) {
    match expr {
        Expression::Ident(ident) => visitor.visit_ident(ident),
        Expression::Literal(literal) => {
            if visitor.visit_literal(literal) {
                match literal {
                    Literal::Array(array_literal) => {
                        for expr in array_literal.contents.iter_mut() {
                            visit_expr(visitor, expr);
                        }
                    }
                    Literal::Slice(array_literal) => {
                        for expr in array_literal.contents.iter_mut() {
                            visit_expr(visitor, expr);
                        }
                    }
                    Literal::Integer(_, _, _)
                    | Literal::Bool(_)
                    | Literal::Unit
                    | Literal::Str(_) => {}
                    Literal::FmtStr(_, _, expr) => {
                        visit_expr(visitor, expr);
                    }
                }
            }
        }
        Expression::Block(expressions) => {
            if visitor.visit_block(expressions) {
                for expr in expressions.iter_mut() {
                    visit_expr(visitor, expr);
                }
            }
        }
        Expression::Unary(unary) => {
            if visitor.visit_unary(unary) {
                visit_expr(visitor, &mut unary.rhs);
            }
        }
        Expression::Binary(binary) => {
            if visitor.visit_binary(binary) {
                visit_expr(visitor, &mut binary.lhs);
                visit_expr(visitor, &mut binary.rhs);
            }
        }
        Expression::Index(index) => {
            if visitor.visit_index(index) {
                visit_expr(visitor, &mut index.collection);
                visit_expr(visitor, &mut index.index);
            }
        }
        Expression::Cast(cast) => {
            if visitor.visit_cast(cast) {
                visit_expr(visitor, &mut cast.lhs);
            }
        }
        Expression::For(for_) => {
            if visitor.visit_for(for_) {
                visit_expr(visitor, &mut for_.start_range);
                visit_expr(visitor, &mut for_.end_range);
                visit_expr(visitor, &mut for_.block);
            }
        }
        Expression::Loop(expr) => {
            if visitor.visit_loop(expr.as_mut()) {
                visit_expr(visitor, expr);
            }
        }
        Expression::While(while_) => {
            if visitor.visit_while(while_) {
                visit_expr(visitor, &mut while_.condition);
                visit_expr(visitor, &mut while_.body);
            }
        }
        Expression::If(if_) => {
            if visitor.visit_if(if_) {
                visit_expr(visitor, &mut if_.condition);
                visit_expr(visitor, &mut if_.consequence);
                if let Some(ref mut alternative) = if_.alternative {
                    visit_expr(visitor, alternative);
                }
            }
        }
        Expression::Match(match_) => {
            if visitor.visit_match(match_) {
                for case in match_.cases.iter_mut() {
                    visit_expr(visitor, &mut case.branch);
                }
                if let Some(ref mut case) = match_.default_case {
                    visit_expr(visitor, case);
                }
            }
        }
        Expression::Tuple(expressions) => {
            if visitor.visit_tuple(expressions) {
                for expr in expressions.iter_mut() {
                    visit_expr(visitor, expr);
                }
            }
        }
        Expression::ExtractTupleField(expression, size) => {
            if visitor.visit_extract_tuple_field(expression, size) {
                visit_expr(visitor, expr);
            }
        }
        Expression::Call(call) => {
            if visitor.visit_call(call) {
                visit_expr(visitor, &mut call.func);
                for arg in call.arguments.iter_mut() {
                    visit_expr(visitor, arg);
                }
            }
        }
        Expression::Let(let_) => {
            if visitor.visit_let(let_) {
                visit_expr(visitor, &mut let_.expression);
            }
        }
        Expression::Constrain(expr, _, _) => {
            if visitor.visit_constrain(expr) {
                visit_expr(visitor, expr);
            }
        }
        Expression::Assign(assign) => {
            if visitor.visit_assign(assign) {
                visit_lvalue(visitor, &mut assign.lvalue);
                visit_expr(visitor, &mut assign.expression);
            }
        }
        Expression::Semi(expr) => {
            if visitor.visit_semi(expr) {
                visit_expr(visitor, expr);
            }
        }
        Expression::Clone(expr) => {
            if visitor.visit_clone(expr) {
                visit_expr(visitor, expr);
            }
        }
        Expression::Drop(expr) => {
            if visitor.visit_drop(expr) {
                visit_expr(visitor, expr);
            }
        }
        Expression::Break => visitor.visit_break(),
        Expression::Continue => visitor.visit_continue(),
    }
}

fn visit_lvalue<V: ExpressionVisitor>(visitor: &mut V, lvalue: &mut LValue) {
    match lvalue {
        LValue::Ident(ident) => visitor.visit_ident(ident),
        LValue::Index { array, index, .. } => {
            visit_lvalue(visitor, array.as_mut());
            visit_expr(visitor, index.as_mut());
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue(visitor, object.as_mut());
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue(visitor, reference.as_mut());
        }
    }
}
