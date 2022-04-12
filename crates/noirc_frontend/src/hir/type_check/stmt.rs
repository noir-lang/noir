use noirc_errors::Span;

use crate::hir_def::stmt::{
    HirAssignStatement, HirConstrainStatement, HirLetStatement, HirPattern, HirStatement, HirLValue,
};
use crate::hir_def::types::Type;
use crate::node_interner::{ExprId, NodeInterner, StmtId};

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

pub fn bind_pattern(
    interner: &mut NodeInterner,
    pattern: &HirPattern,
    typ: Type,
    errors: &mut Vec<TypeCheckError>,
) {
    match pattern {
        HirPattern::Identifier(id) => interner.push_ident_type(id, typ),
        HirPattern::Mutable(pattern, _) => bind_pattern(interner, pattern, typ, errors),
        HirPattern::Tuple(_fields, _span) => {
            todo!("Implement tuple types")
        }
        HirPattern::Struct(struct_type, fields, span) => match typ {
            Type::Struct(_, inner) if &inner == struct_type => {
                let mut pattern_fields = fields.clone();
                let mut type_fields = inner.borrow().fields.clone();

                pattern_fields.sort_by_key(|(id, _)| interner.ident(id));
                type_fields.sort_by_key(|(ident, _)| ident.clone());

                for (pattern_field, type_field) in pattern_fields.into_iter().zip(type_fields) {
                    assert_eq!(interner.ident(&pattern_field.0), type_field.0);
                    bind_pattern(interner, &pattern_field.1, type_field.1, errors);
                }
            }
            Type::Error => (),
            other => {
                errors.push(TypeCheckError::TypeMismatch {
                    expected_typ: other.to_string(),
                    expr_typ: other.to_string(),
                    expr_span: *span,
                });
            }
        },
    }
}

fn type_check_assign_stmt(
    interner: &mut NodeInterner,
    assign_stmt: HirAssignStatement,
    errors: &mut Vec<TypeCheckError>,
) {
    let expr_type = type_check_expression(interner, &assign_stmt.expression, errors);
    let span = interner.expr_span(&assign_stmt.expression);
    let lvalue_type = type_check_lvalue(interner, assign_stmt.lvalue, span, errors);
}

fn type_check_lvalue(
    interner: &mut NodeInterner,
    lvalue: HirLValue,
    assign_span: Span,
    errors: &mut Vec<TypeCheckError>
) -> Type {
    match lvalue {
        HirLValue::Ident(id) => {
            if let Some(ident_def) = interner.ident_def(&id) {
                interner.id_type(&ident_def)
            } else {
                Type::Error
            }
        },
        HirLValue::MemberAccess { object, field_name } => {
            let field_name = interner.ident_name(&field_name);

            let error = |typ| {
                errors.push(TypeCheckError::Unstructured {
                    msg: format!("Type {} has no member named {}", typ, field_name),
                    span: assign_span,
                });
                Type::Error
            };

            match type_check_lvalue(interner, *object, assign_span, errors){
                typ@Type::Struct(_, def) => {
                    if let Some(field) = def.borrow().get_field(&field_name) {
                        field.clone()
                    } else {
                        error(typ)
                    }
                }
                Type::Error => Type::Error,
                other => error(other),
            }
        },
        HirLValue::Index { array, index } => {
            match type_check_lvalue(interner, *array, assign_span, errors){
                Type::Array(_, _, elem_type) => *elem_type,
                Type::Error => Type::Error,
                other => {
                    // TODO: Need a better span here
                    errors.push(TypeCheckError::TypeMismatch {
                        expected_typ: "an array".to_string(),
                        expr_typ: other.to_string(),
                        expr_span: assign_span,
                    });
                    Type::Error
                },
            }
        },
    }
}

fn type_check_let_stmt(
    interner: &mut NodeInterner,
    let_stmt: HirLetStatement,
    errors: &mut Vec<TypeCheckError>,
) {
    let resolved_type =
        type_check_declaration(interner, let_stmt.expression, let_stmt.r#type, errors);

    // Set the type of the pattern to be equal to the annotated type
    bind_pattern(interner, &let_stmt.pattern, resolved_type, errors);
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

/// All declaration statements check that the user specified type(UST) is equal to the
/// expression on the RHS, unless the UST is unspecified in which case
/// the type of the declaration is inferred to match the RHS.
fn type_check_declaration(
    interner: &mut NodeInterner,
    rhs_expr: ExprId,
    mut annotated_type: Type,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    // Type check the expression on the RHS
    let expr_type = type_check_expression(interner, &rhs_expr, errors);

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the expression
    if annotated_type == Type::Unspecified {
        annotated_type = expr_type.clone();
    };

    // Now check if LHS is the same type as the RHS
    // Importantly, we do not co-erce any types implicitly
    if !annotated_type.matches(&expr_type) {
        let expr_span = interner.expr_span(&rhs_expr);
        errors.push(TypeCheckError::TypeMismatch {
            expected_typ: annotated_type.to_string(),
            expr_typ: expr_type.to_string(),
            expr_span,
        });
    }

    expr_type
}
