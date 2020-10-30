use crate::ast::{Statement, Type, PrivateStatement, BlockStatement, Ident, ConstStatement, ConstrainStatement, LetStatement, ArraySize};
use crate::ast::{NoirPath, FunctionDefinition};
use crate::parser::Program;
use crate::{SymbolTable, NoirFunction};
use std::collections::HashMap;

use super::scope::{Scope as GenericScope, ScopeTree as GenericScopeTree, ScopeForest as GenericScopeForest};

type Scope = GenericScope<Ident, Type>;
type ScopeTree = GenericScopeTree<Ident, Type>;
type ScopeForest = GenericScopeForest<Ident, Type>;

use super::errors::{AnalyserError, Span, TypeError};

mod expression;

pub struct TypeChecker<'a> {
    table : &'a SymbolTable,
    local_types : ScopeForest,
    errors : Vec<AnalyserError>

}

impl<'a> TypeChecker<'a> {

    fn from_symbol_table(table : &SymbolTable) -> TypeChecker {
        TypeChecker {table, local_types : ScopeForest::new(), errors:Vec::new()}
    }

    pub fn add_variable_declaration(&mut self, name : Ident, typ : Type) {
        let scope = self.local_types.get_mut_scope();
        let is_new_entry = scope.add_key_value(name.clone(),typ);

        if !is_new_entry {
            panic!("Two parameters in a function cannot have the same name. This should have been caught at the resolver phase")
        }
    }
    pub(super) fn push_err(&mut self, err : impl Into<AnalyserError>) {
        self.errors.push(err.into())
    }

    fn find_function(&self, path : &NoirPath, func_name : &Ident) -> Option<NoirFunction> {   
        self.table.look_up_func(path.clone(), &func_name)
    }
    pub fn lookup_local_identifier(&mut self, name : &Ident) -> Type {
        let scope_tree = self.local_types.current_scope_tree();
        scope_tree.find_key(name).expect("Compiler Error: Cannot find type for specified name. This should be caught by the Resolver pass").clone()
    }

    pub fn check(mut ast : Program, table : &SymbolTable) -> Result<Program, Vec<AnalyserError>> {

        let mut type_checker = TypeChecker::from_symbol_table(table);

        ast.main = match ast.main.clone() {
            Some(main_func) =>    {    
                Some(type_checker.type_check_func_def(main_func))
            },
            None => None
        };

        ast = type_checker.type_check_ast(ast);

        ast.modules = ast.modules.into_iter().map(|(module_id, module)| {
            (module_id, type_checker.type_check_ast(module))
        }).collect();

        if type_checker.errors.len() > 0 {
            return Err(type_checker.errors)
        }

        Ok(ast)
    }

    fn type_check_ast(&mut self, mut ast : Program) -> Program {

        ast.functions = ast.functions.into_iter().map(|func| {
            self.type_check_func_def(func)
        }).collect();

        ast
    }

    // Check that all assignments have the correct types
fn type_check_func_def(&mut self, mut func : FunctionDefinition) -> FunctionDefinition {
    
    self.local_types.start_function();

    // Add function parameters to local types in the type checker
    for param in func.parameters.iter() {
        self.add_variable_declaration(param.0.clone(), param.1.clone());
    }

    let last_return_type = match self.type_check_block_stmt(&mut func.body) {
        Ok(lrt) => lrt,
        Err(_) => return func // If an error is encountered, this will be picked up by the Reporter, lets not pollute stdout with mismatched return type
    };


    let declared_return_type = &func.return_type;

    let is_low_level = match &func.attribute{
        None => false,
        Some(attr) => attr.is_low_level()
    };

    if (&last_return_type != declared_return_type) & !is_low_level {
        let message = format!("mismatched types: Expected the function named `{}` to return the type `{}`, but got `{}`", &func.name.0.contents, declared_return_type, last_return_type);
        let err = AnalyserError::from_ident(message, &func.name);
        self.errors.push(err);
    }

    self.local_types.end_function();

    func
}

// Check that all assignments have the correct types
fn type_check_block_stmt(&mut self, block : &mut BlockStatement) -> Result<Type, AnalyserError> {


    let mut last_return_type = Type::Unit;

    for stmt in block.0.iter_mut() {
       let stmt_return_type =  match stmt {
            Statement::Private(ref mut priv_stmt) => {
                self.type_check_private_stmt(priv_stmt)
            },
            Statement::Expression(ref mut expr) => {
                self.type_check_expr(expr)
            },
            Statement::Block(_) => {
                panic!("Currently we do not support block statements inside of block statements")
            },
            Statement::Const(const_stmt) => {
                self.type_check_const_stmt(const_stmt)
            },
            Statement::If(_) => {
                panic!("[Possible Deprecation] : If statements are not implemented yet, however they might be deprecated for if expressions");
            },
            Statement::Let(let_stmt) => {
                self.type_check_let_stmt(let_stmt)
            },
            Statement::Constrain(constr_stmt) =>{
                self.type_check_constrain_stmt(constr_stmt)
            },
            Statement::Public(_) => {
                // XXX: Initially, we were going to give the ability to declare public variables inside of functions.
                // Now it seems more plausible to only have Public variables be declared as function types,
                // So that we can keep track of linear transformations between public variables which may leak a witness
                //
                // although it is syntax sugaring, it allows users to keep track of public variables, we don't necessarily want them 
                // to be limited to this in the main function parameters
                panic!("[Deprecated] : Declaring public variables in block statements is being deprecated. You will still be able to state them as Types in function parameters ")
            }
        }?;
        // We immediately abort for an error, because we may not have the type information for the identifier 
        // in the statement which threw the error. We could make it more complex in the future, by continuing
        // when the type information is explicitly supplied, and or when it can be inferred from the statement

        last_return_type = stmt_return_type;
    }

    Ok(last_return_type)
}

fn type_check_private_stmt(&mut self,stmt : &mut PrivateStatement) -> Result<Type, AnalyserError>{
    let mut lhs_type = &stmt.r#type; 
    let expr_type = self.type_check_expr(&mut stmt.expression)?;

    // Only witness types can be used in a private statement
    // We additionally enforce that the LHS must be the same type as the RHS 
    // unless the LHS is unspecified. In that case, the LHS will take the type of the RHS

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the RHS
    if lhs_type == &Type::Unspecified {
        lhs_type = &expr_type;
    }

    // Now check if LHS is the same type as the RHS
    // Importantly, we do not co-erce any types implicitly
    if lhs_type != &expr_type {
        let message = format!("\n\nType mismatch: Expected: {:?} got: {:?} \n\n", lhs_type, expr_type);
        return Err(AnalyserError::from_expression(message, &stmt.expression));
    }
    
    // Check if this type can be used in a Private statement
    if !lhs_type.can_be_used_in_priv() {
        let message = format!("The Type {:?} cannot be used in a Private Statement", lhs_type);
        return Err(AnalyserError::from_expression(message, &stmt.expression));
    }

    stmt.r#type = expr_type;

    // Update the Type checker to include this new identifier
    self.add_variable_declaration(stmt.identifier.clone(),stmt.r#type.clone()); 
    Ok(Type::Unit)
  
}
fn type_check_let_stmt(&mut self,stmt : &mut LetStatement) -> Result<Type, AnalyserError>{
    let mut lhs_type = &stmt.r#type; 
    let expr_type = self.type_check_expr(&mut stmt.expression)?;

    // Witness types cannot be used in a let statement, they are for Private statements

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the RHS
    if lhs_type == &Type::Unspecified {
        lhs_type = &expr_type;
    }

    // Now check if LHS is the same type as the RHS
    // Importantly, we do not co-erce any types implicitly
    if lhs_type != &expr_type {
        let message = format!("\n\nType mismatch: Expected: {:?} got: {:?} \n\n", lhs_type, expr_type);
        return Err(AnalyserError::from_expression(message, &stmt.expression));
    }
    
    // Check if this type can be used in a Let statement
    if !lhs_type.can_be_used_in_let() {
        let message = format!("The type {:?} cannot be used in a Let Statement", lhs_type);
        return Err(AnalyserError::from_expression(message, &stmt.expression));
    }
    
    stmt.r#type = expr_type;
    
    // Update the Type checker to include this new identifier
    self.add_variable_declaration(stmt.identifier.clone(),stmt.r#type.clone());

    Ok(Type::Unit)
    
}
fn type_check_const_stmt(&mut self,stmt : &mut ConstStatement) -> Result<Type, AnalyserError>{
    let lhs_type = &stmt.r#type;

    if !(lhs_type == &Type::Constant || lhs_type == &Type::Unspecified) {
        let message = format!("Constant statements can only contain constant types, found type {}", lhs_type);
        return Err(AnalyserError::from_expression(message, &stmt.expression));
    }
    let expr_type = self.type_check_expr(&mut stmt.expression)?;
    
    // Constant statements can only contain the Constant type
    if expr_type != Type::Constant {
        let message = format!("RHS of constrain statement must be of type `Constant`");
        return Err(AnalyserError::from_expression(message, &stmt.expression));
    }
    
    stmt.r#type = expr_type;
    
    // Update the Type checker to include this new identifier
    self.add_variable_declaration(stmt.identifier.clone(),stmt.r#type.clone());
    Ok(Type::Unit)
}
fn type_check_constrain_stmt(&mut self,stmt : &mut ConstrainStatement) -> Result<Type, AnalyserError> {
    let lhs_type = self.type_check_expr(&mut stmt.0.lhs)?;
    let rhs_type = self.type_check_expr(&mut stmt.0.rhs)?;

    // Are there any restrictions on the operator for constrain statements
    if !stmt.0.operator.contents.is_comparator()  {
        let message = format!("Only comparison operators can be used in a constrain statement");
        return Err(AnalyserError::Unstructured{message, span :stmt.0.operator.span()});
    };
    
    if !lhs_type.can_be_used_in_constrain() {
        let message = format!("found type {:?} . This type cannot be used in a constrain statement", lhs_type);
        return Err(AnalyserError::from_expression(message, &stmt.0.lhs));
    }
    if !rhs_type.can_be_used_in_constrain() {
        let message = format!("found type {:?} . This type cannot be used in a constrain statement", rhs_type);
        return Err(AnalyserError::from_expression(message, &stmt.0.rhs));
    }

    Ok(Type::Unit)    // XXX: We leave upper bound checks until runtime, but it is certainly possible to add them to the analyser
}
}



