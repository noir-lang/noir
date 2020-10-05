use super::Resolver;
use super::*;
use libnoirc_ast::{Expression, Ident, Literal, InfixExpression};


impl Resolver {
    pub(crate) fn resolve_expr(&mut self, expr : &Expression) -> bool{
        match expr{
            Expression::Ident(identifier) => self.find_variable(&Ident(identifier.into())),
            Expression::Cast(cast_expr) => {
                self.resolve_expr(&cast_expr.lhs)
            },
            Expression::Call(path,call_expr) => {

                self.find_function(path, &call_expr.func_name);

                self.resolve_list_of_expressions(&call_expr.arguments, "argument", "argument list");
                true
            },
            Expression::Index(index_expr) => {

            let resolved_collection_name = self.find_variable(&index_expr.collection_name);
            let resolved_index = self.resolve_expr(&index_expr.index);

            if !resolved_collection_name {
                panic!("Cannot find a declaration for the array {}", &index_expr.collection_name.0);
            }
            if !resolved_index {
                panic!("Cannot find variable `{:?}` which is being used to index the array {}", &index_expr.index, &index_expr.collection_name.0);
            }
            resolved_collection_name & resolved_index

            },
            Expression::Infix(infix_expr) => {
                self.resolve_infix_expr(&infix_expr, "infix expression")
            },
            Expression::Predicate(pred_expr) => {
                self.resolve_infix_expr(&pred_expr, "predicate expression")
            },
            Expression::Literal(literal) => {
                self.resolve_literal(&literal)
            },
            Expression::If(_) => unimplemented!("[Coming soon] : Currently if expressions have not been implemented"),
            Expression::Prefix(_) => unimplemented!("[Possible Deprecation] : Currently prefix have been rolled back"),
        }
    }

    pub(super) fn resolve_infix_expr(&mut self, infix: &InfixExpression, typ: &str ) -> bool {
        
        let lhs_resolved = self.resolve_expr(&infix.lhs);
        if !lhs_resolved {
            panic!("Could not resolve the lhs of the {} {:?}", typ, infix.lhs);
        }
        
        let rhs_resolved = self.resolve_expr(&infix.rhs);
        if !rhs_resolved {
            panic!("Could not resolve the rhs of the {} {:?}", typ, infix.rhs);
        }

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
            Literal::Func(_) => {
                unimplemented!("[Coming Soon] : Currently function literal have not been fully implemented")
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
                panic!("Cannot resolve the {} at index {} in the {} {:?}", type_of_element, i, data_type, list);
            }
        }
    }
}