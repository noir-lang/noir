use noirc_errors::Span;

use crate::{ArraySize, Type, hir::lower::{HirBinaryOp, HirExpression, HirLiteral, node_interner::{NodeInterner, ExprId, StmtId}, function::Param, stmt::HirStatement}};

use super::errors::TypeCheckError;

pub(crate) fn type_check_expression(interner : &mut NodeInterner, expr_id : &ExprId) -> Result<(), TypeCheckError> {
    let hir_expr = interner.expression(expr_id);
    match hir_expr {
        HirExpression::Ident(ident_id) => {
            // If an Ident is used in an expression, it cannot be a declaration statement  
            let ident_def_id = interner.ident_def(&ident_id).expect("ice: all identifiers should have been resolved. this should have been caught in the resolver");

            // The type of this Ident expression is the type of the Identifier which defined it
            let typ = interner.id_type(ident_def_id);
            interner.push_expr_type(expr_id, typ.clone());
        }
        HirExpression::Literal(literal) => {
            match literal {
                HirLiteral::Array(arr) => {
                    
                    // Type check the contents of the array
                    type_check_list_expression(interner, &arr.contents)?;
                    
                    // Retrieve type for each expression
                    let arr_types : Vec<_> = arr.contents.iter().map(|expr_id| interner.id_type(expr_id)).collect();

                    // Check the result for errors
                    
                    // Specify the type of the Array
                    // Note: This assumes that the array is homogenous, which will be checked next
                    let arr_type = Type::Array(ArraySize::Fixed(arr_types.len() as u128), Box::new(arr_types[0].clone()));
                
                    // Check if the array is homogenous
                    //
                    // An array with one element will be homogenous
                    if arr_types.len() == 1{
                        interner.push_expr_type(expr_id, arr_type);
                        return Ok(())
                    }

                    // To check if an array with more than one element
                    // is homogenous, we can use a sliding window of size two 
                    // to check if adjacent elements are the same
                    // Note: windows(2) expects there to be two or more values
                    // So the case of one element is an edge case which would panic in the compiler.
                    //
                    // XXX: We can refactor this algorithm to peek ahead and check instead of using window.
                    // It would allow us to not need to check the case of one, but it's not significant. 
                    for (index,type_pair) in arr_types.windows(2).enumerate() {
                        let left_type = &type_pair[0]; 
                        let right_type = &type_pair[1]; 

                        if left_type != right_type {
                            let left_span = interner.expr_span(&arr.contents[index]);
                            let right_span = interner.expr_span(&arr.contents[index + 1]);
                            let err = TypeCheckError::NonHomogenousArray{ first_span: left_span, first_type: left_type.to_string(), first_index: index, second_span: right_span, second_type: right_type.to_string(), second_index: index+1};
                            let err = err.add_context("elements in an array must have the same type").unwrap();
                            return Err(err);
                        }
                    }

                    interner.push_expr_type(expr_id, arr_type)
                }
                HirLiteral::Bool(_) => {
                    unimplemented!("currently native boolean types have not been implemented")
                }
                HirLiteral::Integer(_) => {
                    // Literal integers will always be a constant, since the lexer was able to parse the integer
                    interner.push_expr_type(expr_id, Type::Constant);
                }
                HirLiteral::Str(_) => unimplemented!("[Coming Soon] : Currently string literal types have not been implemented"),

            }
        }

        HirExpression::Infix(infix_expr) => {
            // The type of the infix expression must be looked up from a type table
            
            type_check_expression(interner, &infix_expr.lhs)?;
            let lhs_type = interner.id_type(&infix_expr.lhs);
            
            type_check_expression(interner, &infix_expr.rhs)?;
            let rhs_type = interner.id_type(&infix_expr.rhs);

            let result_type = infix_operand_type_rules(&lhs_type,&infix_expr.operator, &rhs_type);
            match result_type {
                Ok(typ) => interner.push_expr_type(expr_id, typ), 
                Err(string) => {
                    let lhs_span = interner.expr_span(&infix_expr.lhs); 
                    let rhs_span = interner.expr_span(&infix_expr.rhs); 
                    return Err(TypeCheckError::Unstructured{ msg: string, span: lhs_span.merge(rhs_span)})
                } 
            }
        }
        HirExpression::Index(index_expr) => {
            let ident_def = interner.ident_def(&index_expr.collection_name).expect("ice : all identifiers should have a def");
            let collection_type = interner.id_type(&ident_def);
            match collection_type {
                // XXX: We can check the array bounds here also, but it may be better to constant fold first
                // and have ConstId instead of ExprId for constants
                Type::Array(_, base_type) => {interner.push_expr_type(expr_id, *base_type)},
                typ => {
                    let span = interner.id_span(&index_expr.collection_name);
                    return Err(TypeCheckError::TypeMismatch{ expected_typ: "Array".to_owned(), expr_typ: typ.to_string(), expr_span: span});
                }
            };

        }
        HirExpression::Call(call_expr) => {
            let func_meta = interner.function_meta(&call_expr.func_id);

            // Check function call arity is correct
            let param_len = func_meta.parameters.len();
            let arg_len = call_expr.arguments.len();
            if param_len != arg_len {
                let span = interner.expr_span(expr_id);
                return Err(TypeCheckError::ArityMisMatch{ expected: param_len as u16, found: arg_len as u16, span});
            }

            // Type check arguments
            let mut arg_types = Vec::with_capacity(call_expr.arguments.len());
            for arg_expr in call_expr.arguments.iter() {
                type_check_expression(interner, arg_expr)?;
                arg_types.push(interner.id_type(arg_expr)) 
            }

            // Check for argument param equality
            for (param, arg) in func_meta.parameters.iter().zip(arg_types) {
                check_param_argument(interner, param, &arg)?
            }

            // The type of the call expression is the return type of the function being called
            interner.push_expr_type(expr_id, func_meta.return_type);
        }
        HirExpression::Cast(cast_expr) => {
            // Evaluate the Lhs
            type_check_expression(interner, &cast_expr.lhs)?;
            let _lhs_type = interner.id_type(cast_expr.lhs);

            // Then check that the type_of(LHS) can be casted to the RHS
            // This is currently being done in the evaluator, we should move it all to here
            // XXX(^) : Move checks for casting from runtime to here

            // type_of(cast_expr) == type_of(cast_type)
            interner.push_expr_type(expr_id, cast_expr.r#type);
        }
        HirExpression::For(for_expr) => {
            type_check_expression(interner, &for_expr.start_range)?;
            type_check_expression(interner, &for_expr.end_range)?;

            let start_range_type = interner.id_type(&for_expr.start_range);
            let end_range_type = interner.id_type(&for_expr.end_range);

            if start_range_type != Type::Constant {
                let span = interner.expr_span(&for_expr.start_range);
                let mut err = TypeCheckError::TypeCannotBeUsed{typ: start_range_type, place: "for loop", span};
                err = err.add_context("currently the range in a loop must be constant literal").unwrap();
                return Err(err)
            }
            if end_range_type != Type::Constant {
                let span = interner.expr_span(&for_expr.end_range);
                let mut err = TypeCheckError::TypeCannotBeUsed{typ: end_range_type, place: "for loop", span};
                err = err.add_context("currently the range in a loop must be constant literal").unwrap();
                return Err(err)
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
            interner.push_expr_type(expr_id, Type::Array(ArraySize::Variable, Box::new(last_type)));
        },
        HirExpression::Block(block_expr) => {
            for stmt in block_expr.statements(){
                super::stmt::type_check(interner, stmt)?
            }
            let last_stmt_type = match block_expr.statements().last() {
                None => Type::Unit,
                Some(stmt) => extract_ret_type(interner, stmt)
            };

            interner.push_expr_type(expr_id, last_stmt_type)

        },
        HirExpression::Prefix(_) => {
            // type_of(prefix_expr) == type_of(rhs_expression)
            todo!("prefix expressions have not been implemented yet")
        },
        HirExpression::Predicate(_) => {todo!("predicate statements have not been implemented yet")},
        HirExpression::If(_) => todo!("If statements have not been implemented yet!"),
    };
    Ok(())
}

    // Given a binary operator and another type. This method will produce the 
    // output type
    pub fn infix_operand_type_rules(lhs_type : &Type, op : &HirBinaryOp, other: &Type) -> Result<Type, String> {
        if op.kind.is_comparator() {
            return Ok(Type::Bool)
        }
        
        match (lhs_type, other)  {

            (Type::Integer(sign_x, bit_width_x), Type::Integer(sign_y, bit_width_y)) => {
                if sign_x != sign_y {
                    return Err(format!("Integers must have the same signedness lhs is {:?}, rhs is {:?} ", sign_x, sign_y))
                }
                if bit_width_x != bit_width_y {
                    return Err(format!("Integers must have the same bit width lhs is {}, rhs is {} ", bit_width_x, bit_width_y))
                }
                Ok(Type::Integer(*sign_x, *bit_width_x))
            }
            (Type::Integer(_, _), Type::Witness) | ( Type::Witness, Type::Integer(_, _) ) => { 
                Err(format!("Cannot use an integer and a witness in a binary operation, try converting the witness into an integer"))
            }
            (Type::Integer(sign_x, bit_width_x), Type::Constant)| (Type::Constant,Type::Integer(sign_x, bit_width_x)) => {
                Ok(Type::Integer(*sign_x, *bit_width_x))
            }
            (Type::Integer(_, _), typ) | (typ,Type::Integer(_, _)) => {
                Err(format!("Integer cannot be used with type {}", typ))
            }

            // Currently, arrays are not supported in binary operations
            (Type::Array(_,_), _) | (_,Type::Array(_, _)) => Err(format!("Arrays cannot be used in an infix operation")),
            
            // An error type on either side will always return an error
            (Type::Error, _) | (_,Type::Error) => Ok(Type::Error),
            (Type::Unspecified, _) | (_,Type::Unspecified) => Ok(Type::Unspecified),
            (Type::Unknown, _) | (_,Type::Unknown) => Ok(Type::Unknown),
            (Type::Unit, _) | (_,Type::Unit) => Ok(Type::Unit),

            // If no side contains an integer. Then we check if either side contains a witness
            // If either side contains a witness, then the final result will be a witness
            (Type::Witness, _) | (_,Type::Witness) => Ok(Type::Witness),
            // Public types are added as witnesses under the hood
            (Type::Public, _) | (_,Type::Public) => Ok(Type::Witness),
            (Type::Bool, _) | (_,Type::Bool) => Ok(Type::Bool),

            (Type::FieldElement, _) | (_,Type::FieldElement) => Ok(Type::FieldElement),
            
            (Type::Constant, Type::Constant)  => Ok(Type::Constant),
        }
        
    }

fn check_param_argument(interner : &NodeInterner, param : &Param, arg_type : &Type) -> Result<(), TypeCheckError>{

        let param_type = &param.1;
        let param_id = param.0;

        if arg_type.is_variable_sized_array() {
            unreachable!("arg type type cannot be a variable sized array. This is not supported.")
        }
        
        // Variable sized arrays (vectors) can be linked to fixed size arrays
        // If the parameter specifies a variable sized array, then we can pass a 
        // fixed size array as an argument
        if param_type.is_variable_sized_array() && arg_type.is_fixed_sized_array() {
            return Ok(())
        }
        
        if param_type != arg_type {
            let span = interner.ident_span(&param_id);
            return Err(TypeCheckError::TypeMismatch{ expected_typ: param_type.to_string(), expr_typ: arg_type.to_string(), expr_span: span});
        }   

        Ok(())
}


fn extract_ret_type(interner : &NodeInterner, stmt_id : &StmtId) -> Type {
    let stmt = interner.statement(stmt_id);
    match stmt {
        HirStatement::Let(_) => Type::Unit,
        HirStatement::Const(_) => Type::Unit,
        HirStatement::Constrain(_) => Type::Unit,
        HirStatement::Public(_) => Type::Unit,
        HirStatement::Private(_) => Type::Unit,
        HirStatement::Expression(expr_id) => interner.id_type(&expr_id) 
    }
}

fn type_check_list_expression(interner: &mut NodeInterner, exprs: &[ExprId]) -> Result<(), TypeCheckError>{
    assert!(exprs.len() > 0);
    
    let (_, errors) : (Vec<_>, Vec<_>) = exprs.into_iter().map(|arg| type_check_expression(interner, arg)).partition(Result::is_ok);

    let errors: Vec<TypeCheckError> = errors.into_iter().map(Result::unwrap_err).collect();
    
    if !errors.is_empty() {
        return Err(TypeCheckError::MultipleErrors(errors));
    }

    Ok(())
}