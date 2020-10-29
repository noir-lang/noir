use super::Resolver;
use super::*;
use crate::ast::{Expression,ExpressionKind, Literal, InfixExpression};


impl<'a> Resolver<'a> {
    pub(crate) fn resolve_expr(&mut self, expr : &Expression) -> bool{
        match &expr.kind{
            ExpressionKind::Ident(identifier) => {
                let resolved = self.find_variable(&identifier.clone().into());
                if !resolved {
                    let err = ResolverError::Unresolved {span :expr.span, symbol_type : "value".to_owned(), symbol : identifier.to_string()};
                    self.push_err(err);
                }
                resolved
            },
            ExpressionKind::Cast(cast_expr) => {
                self.resolve_expr(&cast_expr.lhs)
            },
            ExpressionKind::Call(path,call_expr) => {

                let func = self.find_function(&path, &call_expr.func_name);
                let span = call_expr.func_name.0.span();
                let func = match func {
                    None => {
                        let name = call_expr.func_name.0.contents.clone();
                        let err = ResolverError::Unresolved{span, symbol_type : "function".to_owned(), symbol : name};
                        self.push_err(err);
                        return false
                    },
                    Some(func) => func,
                };

                let param_len = match func{
                    NoirFunction::Function(literal) => literal.parameters.len(),
                    NoirFunction::LowLevelFunction(literal) => literal.parameters.len(),
                };
                let argument_len = call_expr.arguments.len();

                if param_len != argument_len {
                    let message = format!("Function {} expected {} number of arguments, but got {}", call_expr.func_name.0.contents, param_len, argument_len);
                    let err = AnalyserError::from_ident(message, &call_expr.func_name);
                    self.push_err(err);
                }
                
                self.resolve_list_of_expressions(&call_expr.arguments, "argument", "argument list");
                
                true
            },
            ExpressionKind::Index(index_expr) => {
                
                let resolved_collection_name = self.find_variable(&index_expr.collection_name);
                let resolved_index = self.resolve_expr(&index_expr.index);
                
                if !resolved_collection_name {
                    let message = format!("Cannot find a declaration for the array {}", &index_expr.collection_name.0.contents);
                    let err = AnalyserError::from_ident(message, &index_expr.collection_name);
                    self.push_err(err);
                }
                if !resolved_index {
                    let message = format!("Cannot find variable `{:?}` which is being used to index the array {}", &index_expr.index, &index_expr.collection_name.0.contents);
                    let err = AnalyserError::from_ident(message, &index_expr.collection_name);
                    self.push_err(err);
                    
                }
                resolved_collection_name & resolved_index
                
            },
            ExpressionKind::Infix(infix_expr) => {
                self.resolve_infix_expr(&infix_expr)
            },
            ExpressionKind::Predicate(pred_expr) => {
                self.resolve_infix_expr(&pred_expr)
            },
            ExpressionKind::Literal(literal) => {
                self.resolve_literal(&literal)
            },
            ExpressionKind::For(for_expr) => {
                let start_range = &for_expr.start_range;
                let end_range = &for_expr.end_range;
                
                let resolved_lhs = self.resolve_expr(start_range);
                let resolved_rhs = self.resolve_expr(end_range);
                
                if !resolved_lhs {
                    let message = format!("Could not resolve the start range of the for loop");
                    let err = AnalyserError::from_expression(message, &start_range);
                    self.push_err(err)
                }
                if !resolved_rhs {
                    let message = format!("Could not resolve the end range of the for loop");
                    let err = AnalyserError::from_expression(message, &end_range);
                    self.push_err(err)
                }

                self.local_declarations.start_for_loop();
                
                // Add loop identifier
                self.add_variable_decl(for_expr.identifier.clone());
                // Resolve for body
                self.resolve_block_stmt(&for_expr.block);
                
                // Check for unused variables
                let for_scope = self.local_declarations.end_for_loop();
                self.check_for_unused_variables_in_local_scope(&for_scope);

                true
            },
            ExpressionKind::Prefix(_) => unimplemented!("[Possible Deprecation] : Currently prefix have been rolled back"),
        }
    }

    pub(super) fn resolve_infix_expr(&mut self, infix: &InfixExpression) -> bool {
        
        let lhs_resolved = self.resolve_expr(&infix.lhs);    
        let rhs_resolved = self.resolve_expr(&infix.rhs);
    
        lhs_resolved & rhs_resolved
    }

    fn resolve_literal(&mut self, lit: &Literal) -> bool {
        match lit {
            Literal::Array(arr_lit) => {

                self.resolve_list_of_expressions(&arr_lit.contents, "element", "array");
                true
            }, 
            Literal::Bool(_) => {
                unimplemented!("[Coming Soon] : Currently native boolean types have not been implemented")
            }, 
            Literal::Integer(_) => true,
            Literal::Str(_) => {
                unimplemented!("[Coming Soon] : Currently string literal types have not been implemented")
            }, 
            Literal::Type(_) => true
        }
    }

    // type of element refers to the name that each individual element is called.
    // For arrays, this is element. 
    // For a list of arguments, this is argument.
    fn resolve_list_of_expressions(&mut self, list : &Vec<Expression>, type_of_element : &str, data_type : &str ) {
        for (i, element) in list.iter().enumerate() {
            let resolved_element = self.resolve_expr(&element);
            if !resolved_element {
                let message = format!("Cannot resolve the {} at index {} in the {} {:?}", type_of_element, i, data_type, list);
                let err = AnalyserError::from_expression(message, element);
                self.push_err(err);
            }
        }
    }
}