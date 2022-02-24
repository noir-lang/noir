use crate::{
    hir_def::{
        expr::{self, HirBinaryOp, HirExpression, HirLiteral},
        function::Param,
        types::Type,
    },
    node_interner::{ExprId, FuncId, NodeInterner},
    util::vecmap,
    ArraySize, FieldElementType,
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
                    let elem_types = vecmap(&arr.contents, |arg| {
                        type_check_expression(interner, arg, errors)
                    });

                    let first_elem_type = elem_types.get(0).cloned().unwrap_or(Type::Error);

                    // Specify the type of the Array
                    // Note: This assumes that the array is homogeneous, which will be checked next
                    let arr_type = Type::Array(
                        // The FieldElement type is assumed to be private unless the user
                        // adds type annotations that say otherwise.
                        FieldElementType::Private,
                        ArraySize::Fixed(elem_types.len() as u128),
                        Box::new(first_elem_type.clone()),
                    );

                    // Check if the array is homogeneous
                    if first_elem_type != Type::Error {
                        for (index, elem_type) in elem_types.iter().enumerate().skip(1) {
                            if *elem_type != first_elem_type && elem_type != &Type::Error {
                                errors.push(
                                    TypeCheckError::NonHomogeneousArray {
                                        first_span: interner.expr_span(&arr.contents[0]),
                                        first_type: first_elem_type.to_string(),
                                        first_index: index,
                                        second_span: interner.expr_span(&arr.contents[index]),
                                        second_type: elem_type.to_string(),
                                        second_index: index + 1,
                                    }
                                    .add_context("elements in an array must have the same type"),
                                );
                            }
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
            let args = vecmap(&call_expr.arguments, |arg| {
                type_check_expression(interner, arg, errors)
            });
            type_check_function_call(interner, expr_id, &call_expr.func_id, args, errors)
        }
        HirExpression::MethodCall(method_call) => {
            let object_type = type_check_expression(interner, &method_call.object, errors);
            let method_name = method_call.method.0.contents.as_str();
            match lookup_method(interner, object_type.clone(), method_name, expr_id, errors) {
                Some(method_id) => {
                    let mut args = vec![object_type];
                    let mut arg_types = vecmap(&method_call.arguments, |arg| {
                        type_check_expression(interner, arg, errors)
                    });
                    args.append(&mut arg_types);
                    let ret = type_check_function_call(interner, expr_id, &method_id, args, errors);

                    // Desugar the method call into a normal, resolved function call
                    // so that the backend doesn't need to worry about methods
                    let function_call = method_call.into_function_call(method_id);
                    interner.replace_expr(expr_id, function_call);
                    ret
                }
                None => Type::Error,
            }
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
                    .add_context("currently the range in a loop must be constant literal"),
                );
            }

            if end_range_type != Type::FieldElement(FieldElementType::Constant) {
                errors.push(
                    TypeCheckError::TypeCannotBeUsed {
                        typ: end_range_type.clone(),
                        place: "for loop",
                        span: interner.expr_span(&for_expr.end_range),
                    }
                    .add_context("currently the range in a loop must be constant literal"),
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
            let mut block_type = Type::Unit;

            let statements = block_expr.statements();
            for (i, stmt) in statements.iter().enumerate() {
                let expr_type = super::stmt::type_check(interner, stmt, errors);

                if i + 1 < statements.len() {
                    if expr_type != Type::Unit && expr_type != Type::Error {
                        let id = match interner.statement(stmt) {
                            crate::hir_def::stmt::HirStatement::Expression(expr) => expr,
                            _ => *expr_id,
                        };

                        errors.push(TypeCheckError::TypeMismatch {
                            expected_typ: Type::Unit.to_string(),
                            expr_typ: expr_type.to_string(),
                            expr_span: interner.expr_span(&id),
                        });
                    }
                } else {
                    block_type = expr_type;
                }
            }

            block_type
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
        HirExpression::Error => Type::Error,
    };

    interner.push_expr_type(expr_id, typ.clone());
    typ
}

fn lookup_method(
    interner: &mut NodeInterner,
    object_type: Type,
    method_name: &str,
    expr_id: &ExprId,
    errors: &mut Vec<TypeCheckError>,
) -> Option<FuncId> {
    match object_type {
        Type::Struct(_, ref typ) => {
            let typ = typ.borrow();
            match typ.methods.get(method_name) {
                Some(method_id) => Some(*method_id),
                None => {
                    errors.push(TypeCheckError::Unstructured {
                        span: interner.expr_span(expr_id),
                        msg: format!(
                            "No method named '{}' found for type '{}'",
                            method_name, object_type
                        ),
                    });
                    None
                }
            }
        }
        // If we fail to resolve the object to a struct type, we have no way of type
        // checking its arguments as we can't even resolve the name of the function
        Type::Error => None,
        other => {
            errors.push(TypeCheckError::Unstructured {
                span: interner.expr_span(expr_id),
                msg: format!(
                    "Type '{}' must be a struct type to call methods on it",
                    other
                ),
            });
            None
        }
    }
}

fn type_check_function_call(
    interner: &mut NodeInterner,
    expr_id: &ExprId,
    func_id: &FuncId,
    arguments: Vec<Type>,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    if func_id == &FuncId::dummy_id() {
        Type::Error
    } else {
        let func_meta = interner.function_meta(func_id);

        // Check function call arity is correct
        let param_len = func_meta.parameters.len();
        let arg_len = arguments.len();

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
        for (param, arg_type) in func_meta.parameters.iter().zip(arguments) {
            check_param_argument(interner, *expr_id, param, &arg_type, errors);
        }

        // The type of the call expression is the return type of the function being called
        func_meta.return_type
    }
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
        (Integer(..), FieldElement(Private)) | ( FieldElement(Private), Integer(..) ) => {
            Err("Cannot use an integer and a witness in a binary operation, try converting the witness into an integer".to_string())
        }
        (Integer(..), FieldElement(Public)) | ( FieldElement(Public), Integer(..) ) => {
            Err("Cannot use an integer and a public variable in a binary operation, try converting the public into an integer".to_string())
        }
        (Integer(int_field_type,sign_x, bit_width_x), FieldElement(Constant))| (FieldElement(Constant),Integer(int_field_type,sign_x, bit_width_x)) => {
            let field_type = field_type_rules(int_field_type, &Constant);
            Ok(Integer(field_type,*sign_x, *bit_width_x))
        }
        (Integer(..), typ) | (typ,Integer(..)) => {
            Err(format!("Integer cannot be used with type {}", typ))
        }
        // Currently, arrays and structs are not supported in binary operations
        (Array(..), _) | (_, Array(..)) => Err("Arrays cannot be used in an infix operation".to_string()),
        (Struct(..), _) | (_, Struct(..)) => Err("Structs cannot be used in an infix operation".to_string()),

        // An error type on either side will always return an error
        (Error, _) | (_,Error) => Ok(Error),
        (Unspecified, _) | (_,Unspecified) => Ok(Unspecified),
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

                err = err.add_context(context);
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
    assert_eq!(constructor.fields.len(), typ.borrow().fields.len());

    // Sort argument types by name so we can zip with the struct type in the same ordering.
    // Note that we use a Vec to store the original arguments (rather than a BTreeMap) to
    // preserve the evaluation order of the source code.
    let mut args = constructor.fields.clone();
    args.sort_by_key(|arg| interner.ident(&arg.0));

    for ((param_name, param_type), (arg_id, arg)) in typ.borrow().fields.iter().zip(args) {
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

    // TODO: Should a constructor expr always result in a Private type?
    Type::Struct(FieldElementType::Private, typ.clone())
}

pub fn check_member_access(
    access: expr::HirMemberAccess,
    interner: &mut NodeInterner,
    errors: &mut Vec<TypeCheckError>,
) -> Type {
    let lhs_type = type_check_expression(interner, &access.lhs, errors);

    if let Type::Struct(_, s) = &lhs_type {
        let s = s.borrow();
        if let Some(field) = s.fields.iter().find(|(name, _)| name == &access.rhs) {
            // TODO: Should the struct's visibility be applied to the field?
            return field.1.clone();
        }
    }

    if lhs_type != Type::Error {
        errors.push(TypeCheckError::Unstructured {
            msg: format!("Type {} has no member named {}", lhs_type, access.rhs),
            span: interner.expr_span(&access.lhs),
        });
    }

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
        (Integer(..), FieldElement(Private)) | ( FieldElement(Private), Integer(..) ) => {
            Err("Cannot use an integer and a witness in a binary operation, try converting the witness into an integer".to_string())
        }
        (Integer(..), FieldElement(Public)) | ( FieldElement(Public), Integer(..) ) => {
            Err("Cannot use an integer and a public variable in a binary operation, try converting the public into an integer".to_string())
        }
        (Integer(_, _, _), FieldElement(Constant))| (FieldElement(Constant),Integer(_, _, _)) => {
            Ok(Bool)
        }
        (Integer(..), typ) | (typ,Integer(..)) => {
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
