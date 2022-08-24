use noirc_errors::Span;

use crate::hir_def::stmt::{
    HirAssignStatement, HirConstrainStatement, HirLValue, HirLetStatement, HirPattern, HirStatement,
};
use crate::hir_def::types::Type;
use crate::node_interner::{ExprId, NodeInterner, StmtId};
use crate::IsConst;

use super::{errors::TypeCheckError, expr::type_check_expression};

pub(crate) fn type_check(
    interner: &mut NodeInterner,
    stmt_id: &StmtId,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    // println!("inside type_check statement: {:?}", interner.statement(stmt_id));
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
    // println!("in bind_pattern, pattern: {:?}, typ: {:?}", pattern, typ);
    match pattern {
        HirPattern::Identifier(ident) => {
            // println!("bind_pattern, ident.id: {:?}, typ: {:?}", ident.id, typ);
            interner.push_definition_type(ident.id, typ)
        },
        HirPattern::Mutable(pattern, _) => bind_pattern(interner, pattern, typ, errors),
        HirPattern::Tuple(_fields, _span) => {
            todo!("Implement tuple types")
        }
        HirPattern::Struct(struct_type, fields, span) => match typ {
            Type::Struct(inner) if &inner == struct_type => {
                let mut pattern_fields = fields.clone();
                let mut type_fields = inner.borrow().fields.clone();

                pattern_fields.sort_by_key(|(ident, _)| ident.clone());
                type_fields.sort_by_key(|(ident, _)| ident.clone());

                for (pattern_field, type_field) in pattern_fields.into_iter().zip(type_fields) {
                    assert_eq!(&pattern_field.0, &type_field.0);
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

    let span = interner.expr_span(&assign_stmt.expression);
    expr_type.make_subtype_of(&lvalue_type, span, errors, || {
        let msg = format!(
            "Cannot assign an expression of type {} to a value of type {}",
            expr_type, lvalue_type
        );

        TypeCheckError::Unstructured { msg, span }
    });
}

fn type_check_lvalue(
    interner: &mut NodeInterner,
    lvalue: HirLValue,
    assign_span: Span,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    match lvalue {
        HirLValue::Ident(ident) => {
            let definition = interner.definition(ident.id);
            if !definition.mutable {
                errors.push(TypeCheckError::Unstructured {
                    msg: format!("Variable {} must be mutable to be assigned to", definition.name),
                    span: ident.location.span,
                });
            }

            interner.id_type(ident.id)
        }
        HirLValue::MemberAccess { object, field_name } => {
            let result = type_check_lvalue(interner, *object, assign_span, errors);

            let mut error = |typ| {
                errors.push(TypeCheckError::Unstructured {
                    msg: format!("Type {} has no member named {}", typ, field_name),
                    span: field_name.span(),
                });
                Type::Error
            };

            match result {
                Type::Struct(def) => {
                    if let Some(field) = def.borrow().get_field(&field_name.0.contents) {
                        field.clone()
                    } else {
                        error(Type::Struct(def.clone()))
                    }
                }
                Type::Error => Type::Error,
                other => error(other),
            }
        }
        HirLValue::Index { array, index } => {
            let index_type = type_check_expression(interner, &index, errors);
            let expr_span = interner.expr_span(&index);

            index_type.unify(&Type::constant(Some(expr_span)), expr_span, errors, || {
                TypeCheckError::TypeMismatch {
                    expected_typ: "const Field".to_owned(),
                    expr_typ: index_type.to_string(),
                    expr_span,
                }
            });

            match type_check_lvalue(interner, *array, assign_span, errors) {
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
            }
        }
    }
}

fn type_check_let_stmt(
    interner: &mut NodeInterner,
    let_stmt: HirLetStatement,
    errors: &mut Vec<TypeCheckError>,
) {
    // println!("type_check_let_stmt: {:?}", let_stmt);
    let mut resolved_type =
        type_check_declaration(interner, let_stmt.expression, let_stmt.r#type, errors);
    // println!("resolved_type in type_check_let_stmt: {:?}", resolved_type);

    resolved_type.set_const_span(interner.expr_span(&let_stmt.expression));

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

    expr_type.unify(&Type::Bool(IsConst::new(interner)), expr_span, errors, &mut || {
        TypeCheckError::TypeMismatch {
            expr_typ: expr_type.to_string(),
            expected_typ: Type::Bool(IsConst::No(None)).to_string(),
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
    if annotated_type != Type::Unspecified {
        // Now check if LHS is the same type as the RHS
        // Importantly, we do not co-erce any types implicitly
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
