use noirc_errors::Span;

use crate::hir_def::stmt::{
    HirAssignStatement, HirConstrainStatement, HirLValue, HirLetStatement, HirPattern, HirStatement,
};
use crate::hir_def::types::Type;
use crate::node_interner::{DefinitionId, ExprId, NodeInterner, StmtId};
use crate::CompTime;

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
        // it is not bound to anything. We could therefore.
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
        }
        HirStatement::Let(let_stmt) => type_check_let_stmt(interner, let_stmt, errors),
        HirStatement::Constrain(constrain_stmt) => {
            type_check_constrain_stmt(interner, constrain_stmt, errors)
        }
        HirStatement::Assign(assign_stmt) => {
            type_check_assign_stmt(interner, assign_stmt, stmt_id, errors)
        }
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
        HirPattern::Identifier(ident) => interner.push_definition_type(ident.id, typ),
        HirPattern::Mutable(pattern, _) => bind_pattern(interner, pattern, typ, errors),
        HirPattern::Tuple(fields, span) => match typ {
            Type::Tuple(field_types) if field_types.len() == fields.len() => {
                for (field, field_type) in fields.iter().zip(field_types) {
                    bind_pattern(interner, field, field_type, errors);
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
        HirPattern::Struct(struct_type, fields, span) => match typ {
            Type::Struct(inner, args) if &inner == struct_type => {
                let mut pattern_fields = fields.clone();

                pattern_fields.sort_by_key(|(ident, _)| ident.clone());

                for pattern_field in pattern_fields {
                    let type_field =
                        inner.borrow().get_field(&pattern_field.0 .0.contents, &args).unwrap().0;
                    bind_pattern(interner, &pattern_field.1, type_field, errors);
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
    stmt_id: &StmtId,
    errors: &mut Vec<TypeCheckError>,
) {
    let expr_type = type_check_expression(interner, &assign_stmt.expression, errors);
    let span = interner.expr_span(&assign_stmt.expression);
    let (lvalue_type, new_lvalue) = type_check_lvalue(interner, assign_stmt.lvalue, span, errors);

    // Must push new lvalue to the interner, we've resolved any field indices
    interner.update_statement(stmt_id, |stmt| match stmt {
        HirStatement::Assign(assign) => assign.lvalue = new_lvalue,
        _ => unreachable!(),
    });

    let span = interner.expr_span(&assign_stmt.expression);
    expr_type.make_subtype_of(&lvalue_type, span, errors, || {
        let msg = format!(
            "Cannot assign an expression of type {expr_type} to a value of type {lvalue_type}"
        );

        TypeCheckError::Unstructured { msg, span }
    });
}

fn type_check_lvalue(
    interner: &mut NodeInterner,
    lvalue: HirLValue,
    assign_span: Span,
    errors: &mut Vec<TypeCheckError>,
) -> (Type, HirLValue) {
    match lvalue {
        HirLValue::Ident(ident, _) => {
            let typ = if ident.id == DefinitionId::dummy_id() {
                Type::Error
            } else {
                let definition = interner.definition(ident.id);
                if !definition.mutable {
                    errors.push(TypeCheckError::Unstructured {
                        msg: format!(
                            "Variable {} must be mutable to be assigned to",
                            definition.name
                        ),
                        span: ident.location.span,
                    });
                }
                // Do we need to store TypeBindings here?
                interner.id_type(ident.id).instantiate(interner).0
            };

            (typ.clone(), HirLValue::Ident(ident, typ))
        }
        HirLValue::MemberAccess { object, field_name, .. } => {
            let (result, object) = type_check_lvalue(interner, *object, assign_span, errors);
            let object = Box::new(object);

            let mut error = |typ| {
                errors.push(TypeCheckError::Unstructured {
                    msg: format!("Type {typ} has no member named {field_name}"),
                    span: field_name.span(),
                });
                (Type::Error, None)
            };

            let (typ, field_index) = match result {
                Type::Struct(def, args) => {
                    match def.borrow().get_field(&field_name.0.contents, &args) {
                        Some((field, index)) => (field, Some(index)),
                        None => error(Type::Struct(def.clone(), args)),
                    }
                }
                Type::Error => (Type::Error, None),
                other => error(other),
            };

            (typ.clone(), HirLValue::MemberAccess { object, field_name, field_index, typ })
        }
        HirLValue::Index { array, index, .. } => {
            let index_type = type_check_expression(interner, &index, errors);
            let expr_span = interner.expr_span(&index);

            index_type.unify(&Type::comp_time(Some(expr_span)), expr_span, errors, || {
                TypeCheckError::TypeMismatch {
                    expected_typ: "comptime Field".to_owned(),
                    expr_typ: index_type.to_string(),
                    expr_span,
                }
            });

            let (result, array) = type_check_lvalue(interner, *array, assign_span, errors);
            let array = Box::new(array);

            let typ = match result {
                Type::Array(_, elem_type) => *elem_type,
                Type::Error => Type::Error,
                other => {
                    // TODO: Need a better span here
                    errors.push(TypeCheckError::TypeMismatch {
                        expected_typ: "an array".to_string(),
                        expr_typ: other.to_string(),
                        expr_span: assign_span,
                    });
                    Type::Error
                }
            };

            (typ.clone(), HirLValue::Index { array, index, typ })
        }
    }
}

fn type_check_let_stmt(
    interner: &mut NodeInterner,
    let_stmt: HirLetStatement,
    errors: &mut Vec<TypeCheckError>,
) {
    let mut resolved_type =
        type_check_declaration(interner, let_stmt.expression, let_stmt.r#type, errors);

    resolved_type.set_comp_time_span(interner.expr_span(&let_stmt.expression));

    // Set the type of the pattern to be equal to the annotated type
    bind_pattern(interner, &let_stmt.pattern, resolved_type, errors);
}

fn type_check_constrain_stmt(
    interner: &mut NodeInterner,
    stmt: HirConstrainStatement,
    errors: &mut Vec<TypeCheckError>,
) {
    let expr_type = type_check_expression(interner, &stmt.0, errors);
    let expr_span = interner.expr_span(&stmt.0);

    expr_type.unify(&Type::Bool(CompTime::new(interner)), expr_span, errors, || {
        TypeCheckError::TypeMismatch {
            expr_typ: expr_type.to_string(),
            expected_typ: Type::Bool(CompTime::No(None)).to_string(),
            expr_span,
        }
    });
}

/// All declaration statements check that the user specified type(UST) is equal to the
/// expression on the RHS, unless the UST is unspecified in which case
/// the type of the declaration is inferred to match the RHS.
fn type_check_declaration(
    interner: &mut NodeInterner,
    rhs_expr: ExprId,
    annotated_type: Type,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    // Type check the expression on the RHS
    let expr_type = type_check_expression(interner, &rhs_expr, errors);

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the expression
    if annotated_type != Type::Error {
        // Now check if LHS is the same type as the RHS
        // Importantly, we do not coerce any types implicitly
        let expr_span = interner.expr_span(&rhs_expr);
        expr_type.make_subtype_of(&annotated_type, expr_span, errors, || {
            TypeCheckError::TypeMismatch {
                expected_typ: annotated_type.to_string(),
                expr_typ: expr_type.to_string(),
                expr_span,
            }
        });
        annotated_type
    } else {
        expr_type
    }
}
