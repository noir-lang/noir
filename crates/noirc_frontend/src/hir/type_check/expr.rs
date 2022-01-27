use crate::{
    hir_def::{
        expr::{self, HirBinaryOp, HirExpression, HirLiteral},
        function::Param,
        stmt::HirStatement,
    },
    ArraySize, Type,
};
use crate::{
    node_interner::{ExprId, NodeInterner, StmtId},
    FieldElementType,
};

use super::errors::TypeCheckError;

pub(crate) fn type_check_expression(
    interner: &mut NodeInterner,
    expr_id: &ExprId,
) -> Vec<TypeCheckError> {
    let mut errors = vec![];

    match interner.expression(expr_id) {
        HirExpression::Ident(ident_id) => {
            // If an Ident is used in an expression, it cannot be a declaration statement
            if let Some(ident_def_id) = interner.ident_def(&ident_id) {
                // The type of this Ident expression is the type of the Identifier which defined it
                let typ = interner.id_type(ident_def_id);
                interner.push_expr_type(expr_id, typ);
            }
        }
        HirExpression::Literal(literal) => {
            match literal {
                HirLiteral::Array(arr) => {
                    // Type check the contents of the array
                    errors.extend(type_check_list_expression(interner, &arr.contents));

                    // Retrieve type for each expression
                    let arr_types: Vec<_> = arr
                        .contents
                        .iter()
                        .map(|expr_id| interner.id_type(expr_id))
                        .collect();

                    // Check the result for errors

                    // Specify the type of the Array
                    // Note: This assumes that the array is homogeneous, which will be checked next
                    let arr_type = Type::Array(
                        // The FieldElement type is assumed to be private unless the user
                        // adds type annotations that say otherwise.
                        FieldElementType::Private,
                        ArraySize::Fixed(arr_types.len() as u128),
                        Box::new(arr_types[0].clone()),
                    );

                    // Check if the array is homogeneous
                    //
                    // An array with one element will be homogeneous
                    if arr_types.len() == 1 {
                        interner.push_expr_type(expr_id, arr_type);
                        return errors;
                    }

                    // To check if an array with more than one element
                    // is homogeneous, we can use a sliding window of size two
                    // to check if adjacent elements are the same
                    // Note: windows(2) expects there to be two or more values
                    // So the case of one element is an edge case which would panic in the compiler.
                    //
                    // XXX: We can refactor this algorithm to peek ahead and check instead of using window.
                    // It would allow us to not need to check the case of one, but it's not significant.
                    for (index, type_pair) in arr_types.windows(2).enumerate() {
                        let left_type = &type_pair[0];
                        let right_type = &type_pair[1];

                        if left_type != right_type {
                            let left_span = interner.expr_span(&arr.contents[index]);
                            let right_span = interner.expr_span(&arr.contents[index + 1]);
                            errors.push(
                                TypeCheckError::NonHomogeneousArray {
                                    first_span: left_span,
                                    first_type: left_type.to_string(),
                                    first_index: index,
                                    second_span: right_span,
                                    second_type: right_type.to_string(),
                                    second_index: index + 1,
                                }
                                .add_context("elements in an array must have the same type")
                                .unwrap(),
                            );
                        }
                    }

                    interner.push_expr_type(expr_id, arr_type)
                }
                HirLiteral::Bool(_) => {
                    unimplemented!("currently native boolean types have not been implemented")
                }
                HirLiteral::Integer(_) => {
                    // Literal integers will always be a constant, since the lexer was able to parse the integer
                    interner
                        .push_expr_type(expr_id, Type::FieldElement(FieldElementType::Constant));
                }
                HirLiteral::Str(_) => unimplemented!(
                    "[Coming Soon] : Currently string literal types have not been implemented"
                ),
            }
        }

        HirExpression::Infix(infix_expr) => {
            // The type of the infix expression must be looked up from a type table
            errors.extend(type_check_expression(interner, &infix_expr.lhs));
            let lhs_type = interner.id_type(&infix_expr.lhs);

            errors.extend(type_check_expression(interner, &infix_expr.rhs));
            let rhs_type = interner.id_type(&infix_expr.rhs);

            match infix_operand_type_rules(&lhs_type, &infix_expr.operator, &rhs_type) {
                Ok(typ) => interner.push_expr_type(expr_id, typ),
                Err(msg) => {
                    let lhs_span = interner.expr_span(&infix_expr.lhs);
                    let rhs_span = interner.expr_span(&infix_expr.rhs);
                    errors.push(TypeCheckError::Unstructured {
                        msg,
                        span: lhs_span.merge(rhs_span),
                    });
                }
            }
        }
        HirExpression::Index(index_expr) => {
            if let Some(ident_def) = interner.ident_def(&index_expr.collection_name) {
                match interner.id_type(&ident_def) {
                    // XXX: We can check the array bounds here also, but it may be better to constant fold first
                    // and have ConstId instead of ExprId for constants
                    Type::Array(_, _, base_type) => interner.push_expr_type(expr_id, *base_type),
                    typ => {
                        let span = interner.id_span(&index_expr.collection_name);
                        errors.push(TypeCheckError::TypeMismatch {
                            expected_typ: "Array".to_owned(),
                            expr_typ: typ.to_string(),
                            expr_span: span,
                        });
                    }
                }
            }
        }
        HirExpression::Call(call_expr) => {
            let func_meta = interner.function_meta(&call_expr.func_id);

            // Check function call arity is correct
            let param_len = func_meta.parameters.len();
            let arg_len = call_expr.arguments.len();

            // Type check arguments
            let mut arg_types = Vec::with_capacity(call_expr.arguments.len());
            for arg_expr in call_expr.arguments.iter() {
                errors.extend(type_check_expression(interner, arg_expr));
                arg_types.push(interner.id_type(arg_expr))
            }

            if param_len != arg_len {
                let span = interner.expr_span(expr_id);
                errors.push(TypeCheckError::ArityMisMatch {
                    expected: param_len as u16,
                    found: arg_len as u16,
                    span,
                });
            }

            // Check for argument param equality.
            // In the case where we previously issued an error for a parameter count mismatch
            // this will only check up until the shorter of the two Vecs.
            for (param, arg) in func_meta.parameters.iter().zip(arg_types) {
                errors.extend(check_param_argument(interner, *expr_id, param, &arg));
            }

            // The type of the call expression is the return type of the function being called
            interner.push_expr_type(expr_id, func_meta.return_type);
        }
        HirExpression::Cast(cast_expr) => {
            // Evaluate the LHS
            errors.extend(type_check_expression(interner, &cast_expr.lhs));
            let _lhs_type = interner.id_type(cast_expr.lhs);

            // Then check that the type_of(LHS) can be casted to the RHS
            // This is currently being done in the evaluator, we should move it all to here
            // XXX(^) : Move checks for casting from runtime to here

            // type_of(cast_expr) == type_of(cast_type)
            interner.push_expr_type(expr_id, cast_expr.r#type);
        }
        HirExpression::For(for_expr) => {
            errors.extend(type_check_expression(interner, &for_expr.start_range));
            errors.extend(type_check_expression(interner, &for_expr.end_range));

            let start_range_type = interner.id_type(&for_expr.start_range);
            let end_range_type = interner.id_type(&for_expr.end_range);

            if start_range_type != Type::FieldElement(FieldElementType::Constant) {
                errors.push(
                    TypeCheckError::TypeCannotBeUsed {
                        typ: start_range_type.clone(),
                        place: "for loop",
                        span: interner.expr_span(&for_expr.start_range),
                    }
                    .add_context("currently the range in a loop must be constant literal")
                    .unwrap(),
                );
            }

            if end_range_type != Type::FieldElement(FieldElementType::Constant) {
                errors.push(
                    TypeCheckError::TypeCannotBeUsed {
                        typ: end_range_type.clone(),
                        place: "for loop",
                        span: interner.expr_span(&for_expr.end_range),
                    }
                    .add_context("currently the range in a loop must be constant literal")
                    .unwrap(),
                );
            }

            // This check is only needed, if we decide to not have constant range bounds.
            if start_range_type != end_range_type {
                unimplemented!("start range and end range have different types.");
            }
            // The type of the identifier is equal to the type of the ranges
            interner.push_ident_type(&for_expr.identifier, start_range_type);

            errors.extend(type_check_expression(interner, &for_expr.block));
            let last_type = interner.id_type(for_expr.block);
            // XXX: In the release before this, we were using the start and end range to determine the number
            // of iterations and marking the type as Fixed. Is this still necessary?
            // It may be possible to do this properly again, once we do constant folding. Since the range will always be const expr
            interner.push_expr_type(
                expr_id,
                Type::Array(
                    // The type is assumed to be private unless the user specifies
                    // that they want to make it public on the LHS with type annotations
                    FieldElementType::Private,
                    ArraySize::Variable,
                    Box::new(last_type),
                ),
            );
        }
        HirExpression::Block(block_expr) => {
            for stmt in block_expr.statements() {
                errors.extend(super::stmt::type_check(interner, stmt));
            }

            let last_stmt_type = match block_expr.statements().last() {
                None => Type::Unit,
                Some(stmt) => extract_ret_type(interner, stmt),
            };

            interner.push_expr_type(expr_id, last_stmt_type)
        }
        HirExpression::Prefix(_) => {
            // type_of(prefix_expr) == type_of(rhs_expression)
            todo!("prefix expressions have not been implemented yet")
        }
        HirExpression::Predicate(_) => {
            todo!("predicate statements have not been implemented yet")
        }
        HirExpression::If(_) => todo!("If statements have not been implemented yet!"),
        HirExpression::Constructor(constructor) => {
            let typ = &constructor.r#type;
            interner.push_expr_type(expr_id, Type::Struct(typ.clone()));

            // Sanity check, this should be caught during name resolution anyway
            assert_eq!(constructor.fields.len(), typ.fields.len());

            // Sort argument types by name so we can zip with the struct type in the same ordering.
            // Note that we use a Vec to store the original arguments (rather than a BTreeMap) to
            // preserve the evaluation order of the source code.
            let mut args = constructor.fields.clone();
            args.sort_by_key(|arg| interner.ident(&arg.0));

            for ((param_name, param_type), (arg_id, arg)) in typ.fields.iter().zip(args) {
                // Sanity check to ensure we're matching against the same field
                assert_eq!(param_name, &interner.ident(&arg_id));

                errors.extend(type_check_expression(interner, &arg));
                let arg_type = interner.id_type(arg);

                if !param_type.is_super_type_of(&arg_type) {
                    let span = interner.expr_span(expr_id);
                    errors.push(TypeCheckError::TypeMismatch {
                        expected_typ: param_type.to_string(),
                        expr_typ: arg_type.to_string(),
                        expr_span: span,
                    });
                }
            }
        }
        HirExpression::MemberAccess(access) => {
            errors.extend(check_member_access(access, expr_id, interner));
        }
    }

    errors
}

// Given a binary operator and another type. This method will produce the
// output type
// XXX: Review these rules. In particular, the interaction between integers, constants and private/public variables
pub fn infix_operand_type_rules(
    lhs_type: &Type,
    op: &HirBinaryOp,
    other: &Type,
) -> Result<Type, String> {
    if op.kind.is_comparator() {
        return Ok(Type::Bool);
    }

    match (lhs_type, other)  {
        (Type::Integer(lhs_field_type,sign_x, bit_width_x), Type::Integer(rhs_field_type,sign_y, bit_width_y)) => {
            let field_type = field_type_rules(lhs_field_type, rhs_field_type);
            if sign_x != sign_y {
                return Err(format!("Integers must have the same signedness LHS is {:?}, RHS is {:?} ", sign_x, sign_y))
            }
            if bit_width_x != bit_width_y {
                return Err(format!("Integers must have the same bit width LHS is {}, RHS is {} ", bit_width_x, bit_width_y))
            }
            Ok(Type::Integer(field_type,*sign_x, *bit_width_x))
        }
        (Type::Integer(_,_, _), Type::FieldElement(FieldElementType::Private)) | ( Type::FieldElement(FieldElementType::Private), Type::Integer(_,_, _) ) => {
            Err("Cannot use an integer and a witness in a binary operation, try converting the witness into an integer".to_string())
        }
        (Type::Integer(_,_, _), Type::FieldElement(FieldElementType::Public)) | ( Type::FieldElement(FieldElementType::Public), Type::Integer(_,_, _) ) => {
            Err("Cannot use an integer and a public variable in a binary operation, try converting the public into an integer".to_string())
        }
        (Type::Integer(int_field_type,sign_x, bit_width_x), Type::FieldElement(FieldElementType::Constant))| (Type::FieldElement(FieldElementType::Constant),Type::Integer(int_field_type,sign_x, bit_width_x)) => {
            let field_type = field_type_rules(int_field_type, &FieldElementType::Constant);
            Ok(Type::Integer(field_type,*sign_x, *bit_width_x))
        }
        (Type::Integer(_,_, _), typ) | (typ,Type::Integer(_,_, _)) => {
            Err(format!("Integer cannot be used with type {}", typ))
        }
        // Currently, arrays and structs are not supported in binary operations
        (Type::Array(_,_,_), _) | (_,Type::Array(_,_, _)) => Err("Arrays cannot be used in an infix operation".to_string()),
        // Currently, arrays are not supported in binary operations
        (Type::Struct(_), _) | (_, Type::Struct(_)) => Err("Structs cannot be used in an infix operation".to_string()),
        //
        // An error type on either side will always return an error
        (Type::Error, _) | (_,Type::Error) => Ok(Type::Error),
        (Type::Unspecified, _) | (_,Type::Unspecified) => Ok(Type::Unspecified),
        (Type::Unknown, _) | (_,Type::Unknown) => Ok(Type::Unknown),
        (Type::Unit, _) | (_,Type::Unit) => Ok(Type::Unit),
        //
        // If no side contains an integer. Then we check if either side contains a witness
        // If either side contains a witness, then the final result will be a witness
        (Type::FieldElement(FieldElementType::Private), _) | (_,Type::FieldElement(FieldElementType::Private)) => Ok(Type::FieldElement(FieldElementType::Private)),
        // Public types are added as witnesses under the hood
        (Type::FieldElement(FieldElementType::Public), _) | (_,Type::FieldElement(FieldElementType::Public)) => Ok(Type::FieldElement(FieldElementType::Private)),
        (Type::Bool, _) | (_,Type::Bool) => Ok(Type::Bool),
        //
        (Type::FieldElement(FieldElementType::Constant), Type::FieldElement(FieldElementType::Constant))  => Ok(Type::FieldElement(FieldElementType::Constant)),
    }
}

pub fn check_member_access(
    access: expr::HirMemberAccess,
    expr_id: &ExprId,
    interner: &mut NodeInterner,
) -> Vec<TypeCheckError> {
    let mut errors = type_check_expression(interner, &access.lhs);
    let lhs_type = interner.id_type(&access.lhs);

    if let Type::Struct(s) = &lhs_type {
        if let Some(field) = s.fields.iter().find(|(name, _)| name == &access.rhs) {
            interner.push_expr_type(expr_id, field.1.clone());
            return errors;
        }
    }

    errors.push(TypeCheckError::Unstructured {
        msg: format!("Type {} has no member named {}", lhs_type, access.rhs),
        span: interner.expr_span(&access.lhs),
    });
    errors
}

fn field_type_rules(lhs: &FieldElementType, rhs: &FieldElementType) -> FieldElementType {
    match (lhs, rhs) {
        (FieldElementType::Private, FieldElementType::Private) => FieldElementType::Private,
        (FieldElementType::Private, FieldElementType::Public) => FieldElementType::Private,
        (FieldElementType::Private, FieldElementType::Constant) => FieldElementType::Private,
        (FieldElementType::Public, FieldElementType::Private) => FieldElementType::Private,
        (FieldElementType::Public, FieldElementType::Public) => FieldElementType::Public,
        (FieldElementType::Public, FieldElementType::Constant) => FieldElementType::Public,
        (FieldElementType::Constant, FieldElementType::Private) => FieldElementType::Private,
        (FieldElementType::Constant, FieldElementType::Public) => FieldElementType::Public,
        (FieldElementType::Constant, FieldElementType::Constant) => FieldElementType::Constant,
    }
}

fn check_param_argument(
    interner: &NodeInterner,
    expr_id: ExprId,
    param: &Param,
    arg_type: &Type,
) -> Option<TypeCheckError> {
    let param_type = &param.1;

    if arg_type.is_variable_sized_array() {
        unreachable!("arg type type cannot be a variable sized array. This is not supported.")
    }

    (!param_type.is_super_type_of(arg_type)).then(|| TypeCheckError::TypeMismatch {
        expected_typ: param_type.to_string(),
        expr_typ: arg_type.to_string(),
        expr_span: interner.expr_span(&expr_id),
    })
}

fn extract_ret_type(interner: &NodeInterner, stmt_id: &StmtId) -> Type {
    let stmt = interner.statement(stmt_id);
    match stmt {
        HirStatement::Let(_)
        | HirStatement::Const(_)
        | HirStatement::Private(_)
        // We could fetch the type here also for Semi
        // It would return Unit, as we modify the
        // return type in the interner after type checking it
        | HirStatement::Semi(_)
        | HirStatement::Assign(_)
        | HirStatement::Constrain(_) => Type::Unit,
        HirStatement::Expression(expr_id) => interner.id_type(&expr_id),
        HirStatement::Error => Type::Error,
    }
}

fn type_check_list_expression(
    interner: &mut NodeInterner,
    exprs: &[ExprId],
) -> Vec<TypeCheckError> {
    assert!(!exprs.is_empty());

    exprs
        .iter()
        .flat_map(|arg| type_check_expression(interner, arg))
        .collect()
}
