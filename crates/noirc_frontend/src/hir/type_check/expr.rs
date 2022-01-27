use crate::{
    hir_def::{
        expr::{HirBinaryOp, HirExpression, HirLiteral},
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
) -> Result<Type, TypeCheckError> {
    let r#type = match interner.expression(expr_id) {
        HirExpression::Ident(ident_id) => {
            // If an Ident is used in an expression, it cannot be a declaration statement
            let ident_def_id = interner.ident_def(&ident_id).expect("ice: all identifiers should have been resolved. This should have been caught in the resolver");

            // The type of this Ident expression is the type of the Identifier which defined it
            interner.id_type(ident_def_id)
        }
        HirExpression::Literal(literal) => {
            match literal {
                HirLiteral::Array(arr) => {
                    // Type check the contents of the array
                    type_check_list_expression(interner, &arr.contents)?;

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
                        interner.push_expr_type(expr_id, arr_type.clone());
                        return Ok(arr_type);
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
                            let err = TypeCheckError::NonHomogeneousArray {
                                first_span: left_span,
                                first_type: left_type.to_string(),
                                first_index: index,
                                second_span: right_span,
                                second_type: right_type.to_string(),
                                second_index: index + 1,
                            };
                            let err = err
                                .add_context("elements in an array must have the same type")
                                .unwrap();
                            return Err(err);
                        }
                    }

                    arr_type
                }
                HirLiteral::Bool(_) => Type::Bool,
                HirLiteral::Integer(_) => {
                    // Literal integers will always be a constant, since the lexer was able to parse the integer
                    Type::FieldElement(FieldElementType::Constant)
                }
                HirLiteral::Str(_) => unimplemented!(
                    "[Coming Soon] : Currently string literal types have not been implemented"
                ),
            }
        }

        HirExpression::Infix(infix_expr) => {
            // The type of the infix expression must be looked up from a type table

            let lhs_type = type_check_expression(interner, &infix_expr.lhs)?;
            let rhs_type = type_check_expression(interner, &infix_expr.rhs)?;

            let result_type = infix_operand_type_rules(&lhs_type, &infix_expr.operator, &rhs_type);
            match result_type {
                Ok(typ) => typ,
                Err(string) => {
                    let lhs_span = interner.expr_span(&infix_expr.lhs);
                    let rhs_span = interner.expr_span(&infix_expr.rhs);
                    return Err(TypeCheckError::Unstructured {
                        msg: string,
                        span: lhs_span.merge(rhs_span),
                    });
                }
            }
        }
        HirExpression::Index(index_expr) => {
            let ident_def = interner
                .ident_def(&index_expr.collection_name)
                .expect("ice : all identifiers should have a def");

            match interner.id_type(&ident_def) {
                // XXX: We can check the array bounds here also, but it may be better to constant fold first
                // and have ConstId instead of ExprId for constants
                Type::Array(_, _, base_type) => *base_type,
                typ => {
                    let span = interner.id_span(&index_expr.collection_name);
                    return Err(TypeCheckError::TypeMismatch {
                        expected_typ: "Array".to_owned(),
                        expr_typ: typ.to_string(),
                        expr_span: span,
                    });
                }
            }
        }
        HirExpression::Call(call_expr) => {
            let func_meta = interner.function_meta(&call_expr.func_id);

            // Check function call arity is correct
            let param_len = func_meta.parameters.len();
            let arg_len = call_expr.arguments.len();

            if param_len != arg_len {
                let span = interner.expr_span(expr_id);
                return Err(TypeCheckError::ArityMisMatch {
                    expected: param_len as u16,
                    found: arg_len as u16,
                    span,
                });
            }

            // Type check arguments
            let arg_types = call_expr.arguments.iter().map(|arg_expr| {
                type_check_expression(interner, arg_expr)
            }).collect::<Result<Vec<_>, _>>()?;

            // Check for argument param equality
            for (param, arg) in func_meta.parameters.iter().zip(arg_types) {
                check_param_argument(interner, *expr_id, param, &arg)?
            }

            // The type of the call expression is the return type of the function being called
            func_meta.return_type
        }
        HirExpression::Cast(cast_expr) => {
            // Evaluate the LHS
            let _lhs_type = type_check_expression(interner, &cast_expr.lhs)?;

            // Then check that the type_of(LHS) can be casted to the RHS
            // This is currently being done in the evaluator, we should move it all to here
            // XXX(^) : Move checks for casting from runtime to here

            // type_of(cast_expr) == type_of(cast_type)
            cast_expr.r#type
        }
        HirExpression::For(for_expr) => {
            let start_range_type = type_check_expression(interner, &for_expr.start_range)?;
            let end_range_type = type_check_expression(interner, &for_expr.end_range)?;

            if start_range_type != Type::FieldElement(FieldElementType::Constant) {
                let span = interner.expr_span(&for_expr.start_range);
                let mut err = TypeCheckError::TypeCannotBeUsed {
                    typ: start_range_type,
                    place: "for loop",
                    span,
                };
                err = err
                    .add_context("currently the range in a loop must be constant literal")
                    .unwrap();
                return Err(err);
            }
            if end_range_type != Type::FieldElement(FieldElementType::Constant) {
                let span = interner.expr_span(&for_expr.end_range);
                let mut err = TypeCheckError::TypeCannotBeUsed {
                    typ: end_range_type,
                    place: "for loop",
                    span,
                };
                err = err
                    .add_context("currently the range in a loop must be constant literal")
                    .unwrap();
                return Err(err);
            }

            // This check is only needed, if we decide to not have constant range bounds.
            if start_range_type != end_range_type {
                unimplemented!("start range and end range have different types.");
            }
            // The type of the identifier is equal to the type of the ranges
            interner.push_ident_type(&for_expr.identifier, start_range_type);

            type_check_expression(interner, &for_expr.block)?;
            let last_type = interner.id_type(for_expr.block);
            // XXX: In the release before this, we were using the start and end range to determine the number
            // of iterations and marking the type as Fixed. Is this still necessary?
            // It may be possible to do this properly again, once we do constant folding. Since the range will always be const expr
            Type::Array(
                // The type is assumed to be private unless the user specifies
                // that they want to make it public on the LHS with type annotations
                FieldElementType::Private,
                ArraySize::Variable,
                Box::new(last_type),
            )
        }
        HirExpression::Block(block_expr) => {
            for stmt in block_expr.statements() {
                super::stmt::type_check(interner, stmt)?
            }

            match block_expr.statements().last() {
                None => Type::Unit,
                Some(stmt) => extract_ret_type(interner, stmt),
            }
        }
        HirExpression::Prefix(_prefix) => {
            // type_of(prefix_expr) == type_of(rhs_expression)
            todo!("prefix expressions have not been implemented yet")
        }
        HirExpression::Predicate(_predicate) => {
            todo!("predicate statements have not been implemented yet")
        }
        HirExpression::If(if_expr) => {
            let cond_type = type_check_expression(interner, &if_expr.condition)?;
            let then_type = type_check_expression(interner, &if_expr.consequence)?;

            if cond_type != Type::Bool {
                return Err(TypeCheckError::TypeMismatch {
                    expected_typ: Type::Bool.to_string(),
                    expr_typ: cond_type.to_string(),
                    expr_span: interner.expr_span(&if_expr.condition),
                });
            }

            match if_expr.alternative {
                None => Type::Unit,
                Some(alternative) => {
                    let else_type = type_check_expression(interner, &alternative)?;

                    if then_type != else_type {
                        let mut err = TypeCheckError::TypeMismatch {
                            expected_typ: then_type.to_string(),
                            expr_typ: else_type.to_string(),
                            expr_span: interner.expr_span(&alternative),
                        };

                        if then_type == Type::Unit {
                            err = err
                                .add_context("Are you missing a semicolon at the end of your 'else' branch?")
                                .unwrap();
                        } else if else_type == Type::Unit {
                            err = err
                                .add_context("Are you missing a semicolon at the end of your 'then' branch?")
                                .unwrap();
                        } else {
                            err = err
                                .add_context("Expected the types of both if branches to be equal")
                                .unwrap();
                        }

                        return Err(err);
                    }

                    then_type
                }
            }
        },
    };

    interner.push_expr_type(expr_id, r#type.clone());
    Ok(r#type)
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
            // Currently, arrays are not supported in binary operations
            (Type::Array(_,_,_), _) | (_,Type::Array(_,_, _)) => Err("Arrays cannot be used in an infix operation".to_string()),
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
) -> Result<(), TypeCheckError> {
    let param_type = &param.1;

    if arg_type.is_variable_sized_array() {
        unreachable!("arg type type cannot be a variable sized array. This is not supported.")
    }

    if !param_type.is_super_type_of(arg_type) {
        let span = interner.expr_span(&expr_id);
        return Err(TypeCheckError::TypeMismatch {
            expected_typ: param_type.to_string(),
            expr_typ: arg_type.to_string(),
            expr_span: span,
        });
    }

    Ok(())
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
) -> Result<(), TypeCheckError> {
    assert!(!exprs.is_empty());

    let (_, errors): (Vec<_>, Vec<_>) = exprs
        .iter()
        .map(|arg| type_check_expression(interner, arg))
        .partition(Result::is_ok);

    let errors: Vec<TypeCheckError> = errors.into_iter().map(Result::unwrap_err).collect();

    if !errors.is_empty() {
        return Err(TypeCheckError::MultipleErrors(errors));
    }

    Ok(())
}
