use crate::hir_def::stmt::{
    HirAssignStatement, HirConstStatement, HirConstrainStatement, HirLetStatement,
    HirPrivateStatement, HirStatement,
};
use crate::node_interner::{ExprId, NodeInterner, StmtId};
use crate::Type;

use super::{errors::TypeCheckError, expr::type_check_expression};

pub(crate) fn type_check(interner: &mut NodeInterner, stmt_id: &StmtId) -> Vec<TypeCheckError> {
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
        HirStatement::Expression(expr_id) => type_check_expression(interner, &expr_id),
        HirStatement::Semi(expr_id) => {
            let errors = type_check_expression(interner, &expr_id);
            interner.make_expr_type_unit(&expr_id);
            errors
        }
        HirStatement::Private(priv_stmt) => type_check_priv_stmt(interner, priv_stmt),
        HirStatement::Let(let_stmt) => type_check_let_stmt(interner, let_stmt),
        HirStatement::Const(const_stmt) => type_check_const_stmt(interner, const_stmt),
        HirStatement::Constrain(constrain_stmt) => {
            type_check_constrain_stmt(interner, constrain_stmt)
        }
        HirStatement::Assign(assign_stmt) => type_check_assign_stmt(interner, assign_stmt),

        // Avoid issuing further errors for statements that did not parse correctly
        HirStatement::Error => vec![],
    }
}

fn type_check_assign_stmt(
    interner: &mut NodeInterner,
    assign_stmt: HirAssignStatement,
) -> Vec<TypeCheckError> {
    // To get the type of the identifier, we need to get the identifier which defined it
    // once a variable has a type, it cannot be changed
    let ident_def = interner
        .ident_def(&assign_stmt.identifier)
        .expect("all identifiers that define a variable, should have a type during type checking");
    let identifier_type = interner.id_type(&ident_def);

    let mut errors = type_check_expression(interner, &assign_stmt.expression);
    let expr_type = interner.id_type(&assign_stmt.expression);

    if expr_type != identifier_type {
        let expr_span = interner.expr_span(&assign_stmt.expression);
        errors.push(TypeCheckError::TypeMismatch {
            expected_typ: identifier_type.to_string(),
            expr_typ: expr_type.to_string(),
            expr_span,
        });
    }

    errors
}

fn type_check_priv_stmt(
    interner: &mut NodeInterner,
    priv_stmt: HirPrivateStatement,
) -> Vec<TypeCheckError> {
    let (resolved_type, mut errors) =
        type_check_declaration(interner, priv_stmt.expression, priv_stmt.r#type);

    // Check if this type can be used in a Private statement
    if !resolved_type.can_be_used_in_priv() {
        errors.push(TypeCheckError::TypeCannotBeUsed {
            typ: resolved_type.clone(),
            place: "private statement",
            span: interner.expr_span(&priv_stmt.expression),
        });
    }

    // Set the type of the identifier to be equal to the annotated type
    interner.push_ident_type(&priv_stmt.identifier, resolved_type);
    errors
}

fn type_check_let_stmt(
    interner: &mut NodeInterner,
    let_stmt: HirLetStatement,
) -> Vec<TypeCheckError> {
    let (resolved_type, mut errors) =
        type_check_declaration(interner, let_stmt.expression, let_stmt.r#type);

    // Check if this type can be used in a Let statement
    if !resolved_type.can_be_used_in_let() {
        errors.push(TypeCheckError::TypeCannotBeUsed {
            typ: resolved_type.clone(),
            place: "let statement",
            span: interner.expr_span(&let_stmt.expression),
        });
    }

    // Set the type of the identifier to be equal to the annotated type
    interner.push_ident_type(&let_stmt.identifier, resolved_type);
    errors
}

fn type_check_const_stmt(
    interner: &mut NodeInterner,
    const_stmt: HirConstStatement,
) -> Vec<TypeCheckError> {
    // XXX: It may not make sense to have annotations for const statements, since they can only have one type
    // Unless we later want to have u32 constants and check those at compile time.
    let (resolved_type, mut errors) =
        type_check_declaration(interner, const_stmt.expression, const_stmt.r#type);

    if resolved_type != Type::CONSTANT {
        errors.push(
            TypeCheckError::TypeCannotBeUsed {
                typ: resolved_type.clone(),
                place: "constant statement",
                span: interner.expr_span(&const_stmt.expression),
            }
            .add_context("constant statements can only contain constant types")
            .unwrap(),
        );
    }

    interner.push_ident_type(&const_stmt.identifier, resolved_type);
    errors
}

fn type_check_constrain_stmt(
    interner: &mut NodeInterner,
    stmt: HirConstrainStatement,
) -> Vec<TypeCheckError> {
    let mut errors = type_check_expression(interner, &stmt.0.lhs);
    let lhs_type = interner.id_type(stmt.0.lhs);

    errors.extend(type_check_expression(interner, &stmt.0.rhs));
    let rhs_type = interner.id_type(&stmt.0.rhs);

    // Since constrain statements are not expressions, we do not allow predicate or non-comparison binary operators
    if !stmt.0.operator.kind.is_comparator() {
        let span = stmt.0.operator.span;
        let err = TypeCheckError::OpCannotBeUsed {
            op: stmt.0.operator,
            place: "constrain statement",
            span,
        };
        errors.push(
            err.add_context("only comparison operators can be used in a constrain statement")
                .unwrap(),
        );
    };

    if !lhs_type.can_be_used_in_constrain() {
        let span = interner.expr_span(&stmt.0.lhs);
        errors.push(TypeCheckError::TypeCannotBeUsed {
            typ: lhs_type,
            place: "constrain statement",
            span,
        });
    }
    if !rhs_type.can_be_used_in_constrain() {
        let span = interner.expr_span(&stmt.0.rhs);
        errors.push(TypeCheckError::TypeCannotBeUsed {
            typ: rhs_type,
            place: "constrain statement",
            span,
        });
    }

    errors
}

/// All declaration statements check that the user specified type(UST) is equal to the
/// expression on the RHS, unless the UST is unspecified
/// In that case, the UST because the expression
fn type_check_declaration(
    interner: &mut NodeInterner,
    rhs_expr: ExprId,
    mut annotated_type: Type,
) -> (Type, Vec<TypeCheckError>) {
    // Type check the expression on the RHS
    let mut errors = type_check_expression(interner, &rhs_expr);
    let expr_type = interner.id_type(&rhs_expr);

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the expression
    if annotated_type == Type::Unspecified {
        annotated_type = expr_type.clone();
    };

    // Now check if LHS is the same type as the RHS
    // Importantly, we do not co-erce any types implicitly
    if annotated_type != expr_type {
        let expr_span = interner.expr_span(&rhs_expr);
        errors.push(TypeCheckError::TypeMismatch {
            expected_typ: annotated_type.to_string(),
            expr_typ: expr_type.to_string(),
            expr_span,
        });
    }

    // At this point annotated type and user specified type are the same
    // so we can return either. Cloning a Type is Cheap and may eventually be Copy
    (expr_type, errors)
}
