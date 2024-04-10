use iter_extended::vecmap;
use noirc_errors::Spanned;

use crate::{ConstrainKind, LetStatement, Pattern, Ident, Type, Path, UnresolvedType, UnresolvedTypeData};
use crate::node_interner::{NodeInterner, StmtId, ExprId};
use crate::ast::{ Expression, Statement, StatementKind, ConstrainStatement };
use crate::hir_def::expr::{HirExpression, HirIdent};
use crate::hir_def::stmt::{HirStatement, HirPattern};

// TODO:
// - Full path for idents & types
// - Assert/AssertEq information lost
// - The type name span is lost in constructor patterns & expressions
// - All type spans are lost

impl StmtId {
    #[allow(unused)]
    fn to_ast(&self, interner: &NodeInterner) -> Statement {
        let statement = interner.statement(self);
        let span = interner.statement_span(self);
        
        let kind = match statement {
            HirStatement::Let(let_stmt) => {
                let pattern = let_stmt.pattern.to_ast(interner);
                let r#type = interner.id_type(let_stmt.expression).to_ast(interner);
                let expression = let_stmt.expression.to_ast(interner);
                StatementKind::Let(LetStatement { pattern, r#type, expression })
            },
            HirStatement::Constrain(constrain) => {
                let expr = constrain.0.to_ast(interner);
                let message = constrain.2.map(|message| message.to_ast(interner));

                // TODO: Find difference in usage between Assert & AssertEq
                StatementKind::Constrain(ConstrainStatement(expr, message, ConstrainKind::Assert))
            },
            HirStatement::Assign(_) => todo!(),
            HirStatement::For(_) => todo!(),
            HirStatement::Break => todo!(),
            HirStatement::Continue => todo!(),
            HirStatement::Expression(_) => todo!(),
            HirStatement::Semi(_) => todo!(),
            HirStatement::Error => todo!(),
        };

        Statement { kind, span }
    }
}

impl ExprId {
    #[allow(unused)]
    fn to_ast(&self, interner: &NodeInterner) -> Expression {
        let expression = interner.expression(self);
        let location = interner.expr_span(self);

        match expression {
            HirExpression::Ident(_) => todo!(),
            HirExpression::Literal(_) => todo!(),
            HirExpression::Block(_) => todo!(),
            HirExpression::Prefix(_) => todo!(),
            HirExpression::Infix(_) => todo!(),
            HirExpression::Index(_) => todo!(),
            HirExpression::Constructor(_) => todo!(),
            HirExpression::MemberAccess(_) => todo!(),
            HirExpression::Call(_) => todo!(),
            HirExpression::MethodCall(_) => todo!(),
            HirExpression::Cast(_) => todo!(),
            HirExpression::If(_) => todo!(),
            HirExpression::Tuple(_) => todo!(),
            HirExpression::Lambda(_) => todo!(),
            HirExpression::Error => todo!(),
            HirExpression::Quote(_) => todo!(),
        }
    }
}

impl HirPattern {
    fn to_ast(self, interner: &NodeInterner) -> Pattern {
        match self {
            HirPattern::Identifier(ident) => Pattern::Identifier(ident.to_ast(interner)),
            HirPattern::Mutable(pattern, location) => {
                let pattern = Box::new(pattern.to_ast(interner));
                Pattern::Mutable(pattern, location.span, false)
            },
            HirPattern::Tuple(patterns, location) => {
                let patterns = vecmap(patterns, |pattern| pattern.to_ast(interner));
                Pattern::Tuple(patterns, location.span)
            },
            HirPattern::Struct(typ, patterns, location) => {
                let patterns = vecmap(patterns, |(name, pattern)| {
                    (name, pattern.to_ast(interner))
                });
                let name = match typ.follow_bindings() {
                    Type::Struct(struct_def, _) => {
                        let struct_def = struct_def.borrow();
                        struct_def.name.0.contents.clone()
                    },
                    // This pass shouldn't error so if the type isn't a struct we just get a string
                    // representation of any other type and use that. We're relying on name
                    // resolution to fail later when this Ast is re-converted to Hir.
                    other => other.to_string(),
                };
                // The name span is lost here
                let path = Path::from_single(name, location.span);
                Pattern::Struct(path, patterns, location.span)
            },
        }
    }
}

impl HirIdent {
    fn to_ast(&self, interner: &NodeInterner) -> Ident {
        let name = interner.definition_name(self.id).to_owned();
        Ident(Spanned::from(self.location.span, name))
    }
}

impl Type {
    fn to_ast(&self, interner: &NodeInterner) -> UnresolvedType {
        let typ = match self {
            Type::FieldElement => todo!(),
            Type::Array(_, _) => todo!(),
            Type::Slice(_) => todo!(),
            Type::Integer(_, _) => todo!(),
            Type::Bool => todo!(),
            Type::String(_) => todo!(),
            Type::FmtString(_, _) => todo!(),
            Type::Unit => todo!(),
            Type::Tuple(_) => todo!(),
            Type::Struct(_, _) => todo!(),
            Type::Alias(_, _) => todo!(),
            Type::TypeVariable(_, _) => todo!(),
            Type::TraitAsType(_, _, _) => todo!(),
            Type::NamedGeneric(_, _) => todo!(),
            Type::Function(_, _, _) => todo!(),
            Type::MutableReference(_) => todo!(),
            Type::Forall(_, _) => todo!(),
            Type::Constant(_) => todo!(),
            Type::Code => todo!(),
            Type::Error => UnresolvedTypeData::Error,
        };
        UnresolvedType { typ, span: todo!() }
    }
}
