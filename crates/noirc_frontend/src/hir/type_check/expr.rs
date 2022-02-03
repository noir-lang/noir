use crate::{
    hir_def::{
        expr::{self, HirBinaryOp, HirExpression, HirLiteral},
        function::Param,
        stmt::HirStatement,
    },
    util::vecmap,
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
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let typ = match interner.expression(expr_id) {
        HirExpression::Ident(ident_id) => {
            // If an Ident is used in an expression, it cannot be a declaration statement
            match interner.ident_def(&ident_id) {
                Some(ident_def_id) => interner.id_type(ident_def_id),
                None => Type::Error,
            }
        }
        HirExpression::Literal(literal) => {
            match literal {
                HirLiteral::Array(arr) => {
                    // Type check the contents of the array
                    assert!(!arr.contents.is_empty());
                    let elem_types = vecmap(&arr.contents, |arg| {
                        type_check_expression(interner, arg, errors)
                    });

                    // Specify the type of the Array
                    // Note: This assumes that the array is homogeneous, which will be checked next
                    let arr_type = Type::Array(
                        // The FieldElement type is assumed to be private unless the user
                        // adds type annotations that say otherwise.
                        FieldElementType::Private,
                        ArraySize::Fixed(elem_types.len() as u128),
                        Box::new(elem_types[0].clone()),
                    );

                    // Check if the array is homogeneous
                    //
                    // An array with one element will be homogeneous
                    if elem_types.len() == 1 {
                        interner.push_expr_type(expr_id, arr_type.clone());
                        return arr_type;
                    }

                    // To check if an array with more than one element
                    // is homogeneous, we can use a sliding window of size two
                    // to check if adjacent elements are the same
                    // Note: windows(2) expects there to be two or more values
                    // So the case of one element is an edge case which would panic in the compiler.
                    //
                    // XXX: We can refactor this algorithm to peek ahead and check instead of using window.
                    // It would allow us to not need to check the case of one, but it's not significant.
                    for (index, type_pair) in elem_types.windows(2).enumerate() {
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
            let lhs_type = type_check_expression(interner, &infix_expr.lhs, errors);
            let rhs_type = type_check_expression(interner, &infix_expr.rhs, errors);

            match infix_operand_type_rules(&lhs_type, &infix_expr.operator, &rhs_type) {
                Ok(typ) => typ,
                Err(msg) => {
                    let lhs_span = interner.expr_span(&infix_expr.lhs);
                    let rhs_span = interner.expr_span(&infix_expr.rhs);
                    errors.push(TypeCheckError::Unstructured {
                        msg,
                        span: lhs_span.merge(rhs_span),
                    });
                    Type::Error
                }
            }
        }
        HirExpression::Index(index_expr) => {
            if let Some(ident_def) = interner.ident_def(&index_expr.collection_name) {
                match interner.id_type(&ident_def) {
                    // XXX: We can check the array bounds here also, but it may be better to constant fold first
                    // and have ConstId instead of ExprId for constants
                    Type::Array(_, _, base_type) => *base_type,
                    typ => {
                        let span = interner.id_span(&index_expr.collection_name);
                        errors.push(TypeCheckError::TypeMismatch {
                            expected_typ: "Array".to_owned(),
                            expr_typ: typ.to_string(),
                            expr_span: span,
                        });
                        Type::Error
                    }
                }
            } else {
                Type::Error
            }
        }
        HirExpression::Call(call_expr) => {
            let func_meta = interner.function_meta(&call_expr.func_id);

            // Check function call arity is correct
            let param_len = func_meta.parameters.len();
            let arg_len = call_expr.arguments.len();

            if param_len != arg_len {
                let span = interner.expr_span(expr_id);
                errors.push(TypeCheckError::ArityMisMatch {
                    expected: param_len as u16,
                    found: arg_len as u16,
                    span,
                });
            }

            // Check for argument param equality
            // In the case where we previously issued an error for a parameter count mismatch
            // this will only check up until the shorter of the two Vecs.
            // Type check arguments
            for (param, arg) in func_meta.parameters.iter().zip(call_expr.arguments.iter()) {
                let arg_type = type_check_expression(interner, arg, errors);
                check_param_argument(interner, *expr_id, param, &arg_type, errors);
            }

            // The type of the call expression is the return type of the function being called
            func_meta.return_type
        }
        HirExpression::Cast(cast_expr) => {
            // Evaluate the LHS
            let _lhs_type = type_check_expression(interner, &cast_expr.lhs, errors);

            // Then check that the type_of(LHS) can be casted to the RHS
            // This is currently being done in the evaluator, we should move it all to here
            // XXX(^) : Move checks for casting from runtime to here

            // type_of(cast_expr) == type_of(cast_type)
            cast_expr.r#type
        }
        HirExpression::For(for_expr) => {
            let start_range_type = type_check_expression(interner, &for_expr.start_range, errors);
            let end_range_type = type_check_expression(interner, &for_expr.end_range, errors);

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

            let last_type = type_check_expression(interner, &for_expr.block, errors);

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
                super::stmt::type_check(interner, stmt, errors);
            }

            match block_expr.statements().last() {
                None => Type::Unit,
                Some(stmt) => extract_ret_type(interner, stmt),
            }
        }
        HirExpression::Prefix(_) => {
            // type_of(prefix_expr) == type_of(rhs_expression)
            todo!("prefix expressions have not been implemented yet")
        }
        HirExpression::If(if_expr) => check_if_expr(&if_expr, expr_id, interner, errors),
        HirExpression::Constructor(constructor) => {
            check_constructor(&constructor, expr_id, interner, errors)
        }
        HirExpression::MemberAccess(access) => check_member_access(access, interner, errors),
    };

    interner.push_expr_type(expr_id, typ.clone());
    typ
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
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let cond_type = type_check_expression(interner, &if_expr.condition, errors);
    let then_type = type_check_expression(interner, &if_expr.consequence, errors);

    if cond_type != Type::Bool {
        errors.push(TypeCheckError::TypeMismatch {
            expected_typ: Type::Bool.to_string(),
            expr_typ: cond_type.to_string(),
            expr_span: interner.expr_span(&if_expr.condition),
        });
    }

    match if_expr.alternative {
        None => Type::Unit,
        Some(alternative) => {
            let else_type = type_check_expression(interner, &alternative, errors);

            if then_type != else_type {
                let mut err = TypeCheckError::TypeMismatch {
                    expected_typ: then_type.to_string(),
                    expr_typ: else_type.to_string(),
                    expr_span: interner.expr_span(expr_id),
                };

                let context = if then_type == Type::Unit {
                    "Are you missing a semicolon at the end of your 'else' branch?"
                } else if else_type == Type::Unit {
                    "Are you missing a semicolon at the end of the first block of this 'if'?"
                } else {
                    "Expected the types of both if branches to be equal"
                };

                err = err.add_context(context).unwrap();
                errors.push(err);
            }

            then_type
        }
    }
}

fn check_constructor(
    constructor: &expr::HirConstructorExpression,
    expr_id: &ExprId,
    interner: &mut NodeInterner,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
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

        let arg_type = type_check_expression(interner, &arg, errors);

        if !param_type.is_super_type_of(&arg_type) {
            let span = interner.expr_span(expr_id);
            errors.push(TypeCheckError::TypeMismatch {
                expected_typ: param_type.to_string(),
                expr_typ: arg_type.to_string(),
                expr_span: span,
            });
        }
    }

    Type::Struct(typ.clone())
}

pub fn check_member_access(
    access: expr::HirMemberAccess,
    interner: &mut NodeInterner,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let lhs_type = type_check_expression(interner, &access.lhs, errors);

    if let Type::Struct(s) = &lhs_type {
        if let Some(field) = s.fields.iter().find(|(name, _)| name == &access.rhs) {
            return field.1.clone();
        }
    }

    errors.push(TypeCheckError::Unstructured {
        msg: format!("Type {} has no member named {}", lhs_type, access.rhs),
        span: interner.expr_span(&access.lhs),
    });

    Type::Error
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
    errors: &mut Vec<TypeCheckError>,
) {
    let param_type = &param.1;

    if arg_type.is_variable_sized_array() {
        unreachable!("arg type type cannot be a variable sized array. This is not supported.")
    }

    if !param_type.is_super_type_of(arg_type) {
        errors.push(TypeCheckError::TypeMismatch {
            expected_typ: param_type.to_string(),
            expr_typ: arg_type.to_string(),
            expr_span: interner.expr_span(&expr_id),
        });
    }
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
