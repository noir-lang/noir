use crate::ast::{Expression, Type, Literal, InfixExpression, ArraySize};
use super::*;
// We assume that the current symbol table contains the functions metadata
// This is not always the root symbol table,if we do not look up the path from the root, if we introduce the closures
// In that case, it would be the functions symbol table

impl<'a> TypeChecker<'a> {
    pub fn type_check_expr(&mut self,expr :  &mut Expression) -> Type {
        match expr {
            Expression::Cast(cast) => cast.r#type.clone() ,
            Expression::Call(path, call_expr) => {

                // Find function
                let func = self.find_function(path, &call_expr.func_name);
                let func = match func {
                    None => panic!("Could not find a function named {} , under the path {:?}", &call_expr.func_name.0, path),
                    Some(func) => func,
                };

                let (parameters, return_type) = match func {
                    NoirFunction::LowLevelFunction(literal) => (literal.parameters, literal.return_type),
                    NoirFunction::Function(literal) => (literal.parameters, literal.return_type),
                };

                let argument_types : Vec<Type> = call_expr.arguments.iter().map(|arg| self.type_check_expr(&mut arg.clone())).collect();

                assert_eq!(parameters.len(), argument_types.len()); // This should have been caught in the resolver

                for (parameter, argument_type) in parameters.iter().zip(argument_types.iter()) {
                    TypeChecker::type_check_param_argument(parameter, argument_type)
                }

               return_type
            },
            Expression::Ident(iden) => {
                self.lookup_local_identifier(&iden.to_string().into())
            },
            Expression::Literal(ref mut lit) => self.type_check_literal(lit),
            Expression::Infix(ref mut infx) => self.type_check_infix(infx),
            Expression::Predicate(ref mut infx) => self.type_check_infix(infx),
            Expression::Index(indx) => {
                // Currently we only index in Arrays and arrays need to have homogenous types
                
                // Find the type for the identifier
                let typ = self.lookup_local_identifier(&indx.collection_name);

                let (_, base_type) = match typ {
                    Type::Array(num_elements, base_type) => (num_elements, base_type),
                    _=> panic!("Cannot index on non array types")
                };
                *base_type
            },
            Expression::For(for_expr) => {
                let start_range = &mut for_expr.start_range;
                let end_range = &mut for_expr.end_range;

                
                let start_type = self.type_check_expr(start_range);
                let end_type = self.type_check_expr(end_range);
                
                assert_eq!(start_type, Type::Constant);
                assert_eq!(end_type, Type::Constant);
                
                self.local_types.start_for_loop();
                
                self.add_variable_declaration(for_expr.identifier.clone(), Type::Constant);
                
                // Note, we ignore return type in a block statement for a for-loop
                let base_typ = self.type_check_block_stmt(&mut for_expr.block);
                self.local_types.end_for_loop();

                // Try to figure out the number of iterations
                let num_iterations = TypeChecker::count_num_of_iterations(&start_range, end_range);
                let array_size = match num_iterations {
                    Some(integer) => ArraySize::Fixed(integer),
                    None => ArraySize::Variable,
                };
                Type::Array(array_size, Box::new(base_typ))
            },
            Expression::Prefix(_) => unimplemented!("[Possible Deprecation] : Currently prefix have been rolled back")
        }
    }
    
    fn type_check_literal(&mut self,lit : &mut Literal) -> Type{
        match lit {
            Literal::Array(arr_lit) => {
                // Arrays are parsed with unspecified types, so they need to be correctly typed here
                //
                // First collect each elements type
                let arr_types : Vec<_> = arr_lit.contents.iter_mut().map(|element| self.type_check_expr(element)).collect();
                if arr_types.len() == 0 {
                    arr_lit.r#type = Type::Unit;
                    return Type::Unit;
                }

                // Specify the type of the Array
                arr_lit.r#type = Type::Array(ArraySize::Fixed(arr_types.len() as u128), Box::new(arr_types[0].clone()));
                
                // Check if the array is homogenous
                
                if arr_types.len() == 1{
                    return arr_lit.r#type.clone()
                }

                for (i,type_pair) in arr_types.windows(2).enumerate() {
                    let left_type = &type_pair[0]; 
                    let right_type = &type_pair[1]; 

                    if left_type != right_type {
                        panic!("Array is not homogenous at indices ({}, {}), found an element of type {} and an element of type {}", i,i+1, left_type, right_type)
                    }
                }
                
                return arr_lit.r#type.clone()
            }, 
            Literal::Bool(_) => {
                unimplemented!("[Coming Soon] : Currently native boolean types have not been implemented")
            }, 
            Literal::Integer(_) => {
                // Literal integers will always be a constant, since the lexer was able to parse the integer
                return Type::Constant;
            },
            Literal::Str(_) => {
                unimplemented!("[Coming Soon] : Currently string literal types have not been implemented")
            }, 
            Literal::Type(typ) => typ.clone()
        }
    }
    
    pub fn type_check_infix(&mut self,infx: &mut InfixExpression) -> Type {
        
        let lhs_type = self.type_check_expr(&mut infx.lhs);
        let rhs_type = self.type_check_expr(&mut infx.rhs);
    
        lhs_type.infix_operand_type_rules(&infx.operator, &rhs_type)
    }

    fn type_check_param_argument(param: &(Ident, Type), arg_type : &Type) {

        let param_name = &param.0;
        let param_type = &param.1;

        if arg_type.is_variable_sized_array() {
            panic!("arg_type type cannot be a variable sized array")
        }
        
        // Variable sized arrays (vectors) can be linked to fixed size arrays
        if param_type.is_variable_sized_array() && arg_type.is_fixed_sized_array() {
            return
        }
        
        if param_type != arg_type {
            panic!("Expected {} for parameter {} but got {} ", param_type, param_name.0, arg_type)
        }
        
    }

    fn extract_constant(expr : &Expression) -> Option<noir_field::FieldElement> {
        let literal = match expr {
            Expression::Literal(literal) => literal,
            _ => return None
        };

        let integer = match literal {
            Literal::Integer(integer) => integer,
            _ => return None
        };

        Some((*integer).into())
    }

    // This is a very naive way to get the amount of loop iterations and 
    // it only works if constant literals are used.
    // Since the Type checker does not have the value of identifiers only their types
    fn count_num_of_iterations(start : &Expression, end : &Expression) -> Option<u128> {
        
        let start_constant = TypeChecker::extract_constant(start)?;
        let end_constant = TypeChecker::extract_constant(end)?;

        let num_iterations = end_constant - start_constant;
        
        Some(num_iterations.to_u128())
    }

}

