use crate::{
    hir_def::{
        expr::{self, HirBinaryOp, HirExpression, HirLiteral},
        function::Param,
    },
    util::vecmap,
    ArraySize, Type,
};
use crate::{
    node_interner::{ExprId, NodeInterner},
    FieldElementType,
};

use super::errors::TypeCheckError;

pub(crate) fn type_check_expression(
    interner: &mut NodeInterner,
    expr_id: &ExprId,
) -> Result<Type, TypeCheckError> {
    let typ = match interner.expression(expr_id) {
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
                    let arr_types = vecmap(&arr.contents, |expr_id| interner.id_type(expr_id));

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

            match infix_operand_type_rules(&lhs_type, &infix_expr.operator, &rhs_type) {
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
            let mut arg_types = Vec::with_capacity(call_expr.arguments.len());
            for arg_expr in call_expr.arguments.iter() {
                arg_types.push(type_check_expression(interner, arg_expr)?);
            }

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

            let last_type = type_check_expression(interner, &for_expr.block)?;

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
            let mut last = None;

            let statements = block_expr.statements();
            for (i, stmt) in statements.iter().enumerate() {
                let expr_type = super::stmt::type_check(interner, stmt)?;

                if i + 1 < statements.len() {
                    if expr_type != Type::Unit {
                        return Err(TypeCheckError::TypeMismatch {
                            expected_typ: Type::Unit.to_string(),
                            expr_typ: expr_type.to_string(),
                            expr_span: interner.expr_span(expr_id),
                        });
                    }
                } else {
                    last = Some(expr_type);
                }
            }

            last.unwrap_or(Type::Unit)
        }
        HirExpression::Prefix(_) => {
            // type_of(prefix_expr) == type_of(rhs_expression)
            todo!("prefix expressions have not been implemented yet")
        }
        HirExpression::If(if_expr) => check_if_expr(&if_expr, expr_id, interner)?,
        HirExpression::Constructor(constructor) => {
            check_constructor(&constructor, expr_id, interner)?
        }
        HirExpression::MemberAccess(access) => check_member_access(access, interner)?,
    };

    interner.push_expr_type(expr_id, typ.clone());
    Ok(typ)
}

// Given a binary operator and another type. This method will produce the output type
// XXX: Review these rules. In particular, the interaction between integers, constants and private/public variables
pub fn infix_operand_type_rules(
    lhs_type: &Type,
    op: &HirBinaryOp,
    other: &Type,
) -> Result<Type, String> {
    if op.kind.is_comparator() {
        return comparator_operand_type_rules(lhs_type, other);
    }

    use {FieldElementType::*, Type::*};
    match (lhs_type, other)  {
        (Integer(lhs_field_type, sign_x, bit_width_x), Integer(rhs_field_type, sign_y, bit_width_y)) => {
            let field_type = field_type_rules(lhs_field_type, rhs_field_type);
            if sign_x != sign_y {
                return Err(format!("Integers must have the same signedness LHS is {:?}, RHS is {:?} ", sign_x, sign_y))
            }
            if bit_width_x != bit_width_y {
                return Err(format!("Integers must have the same bit width LHS is {}, RHS is {} ", bit_width_x, bit_width_y))
            }
            Ok(Integer(field_type, *sign_x, *bit_width_x))
        }
        (Integer(_,_, _), FieldElement(Private)) | ( FieldElement(Private), Integer(_,_, _) ) => {
            Err("Cannot use an integer and a witness in a binary operation, try converting the witness into an integer".to_string())
        }
        (Integer(_,_, _), FieldElement(Public)) | ( FieldElement(Public), Integer(_,_, _) ) => {
            Err("Cannot use an integer and a public variable in a binary operation, try converting the public into an integer".to_string())
        }
        (Integer(int_field_type,sign_x, bit_width_x), FieldElement(Constant))| (FieldElement(Constant),Integer(int_field_type,sign_x, bit_width_x)) => {
            let field_type = field_type_rules(int_field_type, &Constant);
            Ok(Integer(field_type,*sign_x, *bit_width_x))
        }
        (Integer(_,_, _), typ) | (typ,Integer(_,_, _)) => {
            Err(format!("Integer cannot be used with type {}", typ))
        }
        // Currently, arrays and structs are not supported in binary operations
        (Array(_,_,_), _) | (_, Array(_,_,_)) => Err("Arrays cannot be used in an infix operation".to_string()),
        (Struct(_), _) | (_, Struct(_)) => Err("Structs cannot be used in an infix operation".to_string()),

        // An error type on either side will always return an error
        (Error, _) | (_,Error) => Ok(Error),
        (Unspecified, _) | (_,Unspecified) => Ok(Unspecified),
        (Unknown, _) | (_,Unknown) => Ok(Unknown),
        (Unit, _) | (_,Unit) => Ok(Unit),
        //
        // If no side contains an integer. Then we check if either side contains a witness
        // If either side contains a witness, then the final result will be a witness
        (FieldElement(Private), _) | (_,FieldElement(Private)) => Ok(FieldElement(Private)),
        // Public types are added as witnesses under the hood
        (FieldElement(Public), _) | (_,FieldElement(Public)) => Ok(FieldElement(Private)),
        (Bool, _) | (_,Bool) => Ok(Bool),
        //
        (FieldElement(Constant), FieldElement(Constant))  => Ok(FieldElement(Constant)),
    }
}

fn check_if_expr(
    if_expr: &expr::HirIfExpression,
    expr_id: &ExprId,
    interner: &mut NodeInterner,
) -> Result<Type, TypeCheckError> {
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
        None => Ok(Type::Unit),
        Some(alternative) => {
            let else_type = type_check_expression(interner, &alternative)?;

            if then_type != else_type {
                let mut err = TypeCheckError::TypeMismatch {
                    expected_typ: then_type.to_string(),
                    expr_typ: else_type.to_string(),
                    expr_span: interner.expr_span(expr_id),
                };

                if then_type == Type::Unit {
                    err = err
                        .add_context(
                            "Are you missing a semicolon at the end of your 'else' branch?",
                        )
                        .unwrap();
                } else if else_type == Type::Unit {
                    err = err
                        .add_context(
                            "Are you missing a semicolon at the end of the first block of this 'if'?",
                        )
                        .unwrap();
                } else {
                    err = err
                        .add_context("Expected the types of both if branches to be equal")
                        .unwrap();
                }

                return Err(err);
            }

            Ok(then_type)
        }
    }
}

fn check_constructor(
    constructor: &expr::HirConstructorExpression,
    expr_id: &ExprId,
    interner: &mut NodeInterner,
) -> Result<Type, TypeCheckError> {
    let typ = &constructor.r#type;

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

        type_check_expression(interner, &arg)?;
        let arg_type = interner.id_type(arg);

        if !param_type.is_super_type_of(&arg_type) {
            let span = interner.expr_span(expr_id);
            return Err(TypeCheckError::TypeMismatch {
                expected_typ: param_type.to_string(),
                expr_typ: arg_type.to_string(),
                expr_span: span,
            });
        }
    }

    Ok(Type::Struct(typ.clone()))
}

pub fn check_member_access(
    access: expr::HirMemberAccess,
    interner: &mut NodeInterner,
) -> Result<Type, TypeCheckError> {
    let lhs_type = type_check_expression(interner, &access.lhs)?;

    if let Type::Struct(s) = &lhs_type {
        if let Some(field) = s.fields.iter().find(|(name, _)| name == &access.rhs) {
            return Ok(field.1.clone());
        }
    }

    Err(TypeCheckError::Unstructured {
        msg: format!("Type {} has no member named {}", lhs_type, access.rhs),
        span: interner.expr_span(&access.lhs),
    })
}

fn field_type_rules(lhs: &FieldElementType, rhs: &FieldElementType) -> FieldElementType {
    use FieldElementType::*;
    match (lhs, rhs) {
        (Private, Private) => Private,
        (Private, Public) => Private,
        (Private, Constant) => Private,
        (Public, Private) => Private,
        (Public, Public) => Public,
        (Public, Constant) => Public,
        (Constant, Private) => Private,
        (Constant, Public) => Public,
        (Constant, Constant) => Constant,
    }
}

pub fn comparator_operand_type_rules(lhs_type: &Type, other: &Type) -> Result<Type, String> {
    use {FieldElementType::*, Type::*};
    match (lhs_type, other)  {
        (Integer(_, sign_x, bit_width_x), Integer(_, sign_y, bit_width_y)) => {
            if sign_x != sign_y {
                return Err(format!("Integers must have the same signedness LHS is {:?}, RHS is {:?} ", sign_x, sign_y))
            }
            if bit_width_x != bit_width_y {
                return Err(format!("Integers must have the same bit width LHS is {}, RHS is {} ", bit_width_x, bit_width_y))
            }
            Ok(Bool)
        }
        (Integer(_,_, _), FieldElement(Private)) | ( FieldElement(Private), Integer(_,_, _) ) => {
            Err("Cannot use an integer and a witness in a binary operation, try converting the witness into an integer".to_string())
        }
        (Integer(_,_, _), FieldElement(Public)) | ( FieldElement(Public), Integer(_,_, _) ) => {
            Err("Cannot use an integer and a public variable in a binary operation, try converting the public into an integer".to_string())
        }
        (Integer(_, _, _), FieldElement(Constant))| (FieldElement(Constant),Integer(_, _, _)) => {
            Ok(Bool)
        }
        (Integer(_,_, _), typ) | (typ,Integer(_,_, _)) => {
            Err(format!("Integer cannot be used with type {}", typ))
        }
        // If no side contains an integer. Then we check if either side contains a witness
        // If either side contains a witness, then the final result will be a witness
        (FieldElement(Private), FieldElement(_)) | (FieldElement(_), FieldElement(Private)) => Ok(Bool),
        // Public types are added as witnesses under the hood
        (FieldElement(Public), FieldElement(_)) | (FieldElement(_), FieldElement(Public)) => Ok(Bool),
        (FieldElement(Constant), FieldElement(Constant))  => Ok(Bool),

        // <= and friends are technically valid for booleans, just not very useful
        (Bool, Bool) => Ok(Bool),

        // Avoid reporting errors multiple times
        (Error, _) | (_,Error) => Ok(Error),
        (Unspecified, _) | (_,Unspecified) => Ok(Unspecified),
        (Unknown, _) | (_,Unknown) => Ok(Unknown),
        (lhs, rhs) => Err(format!("Unsupported types for comparison: {} and {}", lhs, rhs)),
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

fn type_check_list_expression(
    interner: &mut NodeInterner,
    exprs: &[ExprId],
) -> Result<(), TypeCheckError> {
    assert!(!exprs.is_empty());

    let (_, errors): (Vec<_>, Vec<_>) = exprs
        .iter()
        .map(|arg| type_check_expression(interner, arg))
        .partition(Result::is_ok);

    let errors = vecmap(errors, Result::unwrap_err);

    if !errors.is_empty() {
        return Err(TypeCheckError::MultipleErrors(errors));
    }

    Ok(())
}
