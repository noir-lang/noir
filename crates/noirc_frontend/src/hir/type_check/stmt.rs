use crate::hir_def::stmt::{
    HirAssignStatement, HirConstrainStatement, HirLetStatement, HirStatement,
};
use crate::node_interner::{NodeInterner, StmtId};
use crate::Type;

use super::{errors::TypeCheckError, expr::type_check_expression};

pub(crate) fn type_check(
    interner: &mut NodeInterner,
    stmt_id: &StmtId,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    match interner.statement(stmt_id) {
        // Lets lay out a convincing argument that the handling of
        // SemiExpressions and Expressions below is correct.
        //
        // The only time you will get a Semi expression is if
        // you have an expression by itself
        //
        // Example:
        //
        // 5; or x; or x+a;
        //
        // In these cases, you cannot even get the expr_id because
        // it is not binded to anything. We could therefore.
        //
        // However since TypeChecking checks the return type of the last statement
        // the type checker could in the future incorrectly return the type.
        //
        // As it stands, this is also impossible because the ret_type function
        // does not use the interner to get the type. It returns Unit.
        //
        // The reason why we still modify the database, is to make sure it is future-proof
        HirStatement::Expression(expr_id) => {
            return type_check_expression(interner, &expr_id, errors);
        }
        HirStatement::Semi(expr_id) => {
            type_check_expression(interner, &expr_id, errors);
            interner.make_expr_type_unit(&expr_id);
        }
        HirStatement::Let(let_stmt) => type_check_let_stmt(interner, let_stmt, errors),
        HirStatement::Constrain(constrain_stmt) => {
            type_check_constrain_stmt(interner, constrain_stmt, errors)
        }
        HirStatement::Assign(assign_stmt) => type_check_assign_stmt(interner, assign_stmt, errors),
        HirStatement::Error => (),
    }
    Type::Unit
}

fn type_check_assign_stmt(
    interner: &mut NodeInterner,
    assign_stmt: HirAssignStatement,
    errors: &mut Vec<TypeCheckError>,
) {
    let expr_type = type_check_expression(interner, &assign_stmt.expression, errors);

    // To get the type of the identifier, we need to get the identifier which defined it
    // once a variable has a type, it cannot be changed
    if let Some(ident_def) = interner.ident_def(&assign_stmt.identifier) {
        let identifier_type = interner.id_type(&ident_def);

        if expr_type != identifier_type {
            errors.push(TypeCheckError::TypeMismatch {
                expected_typ: identifier_type.to_string(),
                expr_typ: expr_type.to_string(),
                expr_span: interner.expr_span(&assign_stmt.expression),
            });
        }
    }
}

fn type_check_let_stmt(
    interner: &mut NodeInterner,
    let_stmt: HirLetStatement,
    errors: &mut Vec<TypeCheckError>,
) {
    // Type check the expression on the RHS
    let expr_type = type_check_expression(interner, &let_stmt.expression, errors);

    let mut resolved_type = let_stmt.r#type;

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the expression
    if resolved_type == Type::Unspecified {
        resolved_type = expr_type.clone();
    };

    // Now check if LHS is the same type as the RHS
    // Importantly, we do not co-erce any types implicitly
    if resolved_type != expr_type {
        let expr_span = interner.expr_span(&let_stmt.expression);
        errors.push(TypeCheckError::TypeMismatch {
            expected_typ: resolved_type.to_string(),
            expr_typ: expr_type.to_string(),
            expr_span,
        });
    }

    // Set the type of the identifier to be equal to the resolved type
    interner.push_ident_type(&let_stmt.identifier, resolved_type);
}

fn type_check_constrain_stmt(
    interner: &mut NodeInterner,
    stmt: HirConstrainStatement,
    errors: &mut Vec<TypeCheckError>,
) {
    let lhs_type = type_check_expression(interner, &stmt.0.lhs, errors);
    let rhs_type = type_check_expression(interner, &stmt.0.rhs, errors);

    // Since constrain statements are not expressions, we disallow non-comparison binary operators
    if !stmt.0.operator.kind.is_comparator() {
        errors.push(
            TypeCheckError::OpCannotBeUsed {
                op: stmt.0.operator,
                place: "constrain statement",
                span: stmt.0.operator.span,
            }
            .add_context("only comparison operators can be used in a constrain statement"),
        );
    };

    if !lhs_type.can_be_used_in_constrain() {
        errors.push(TypeCheckError::TypeCannotBeUsed {
            typ: lhs_type,
            place: "constrain statement",
            span: interner.expr_span(&stmt.0.lhs),
        });
    }

    if !rhs_type.can_be_used_in_constrain() {
        errors.push(TypeCheckError::TypeCannotBeUsed {
            typ: rhs_type,
            place: "constrain statement",
            span: interner.expr_span(&stmt.0.rhs),
        });
    }
}
