use std::collections::HashMap;

use crate::{Path, hir::{Context, basic_data::FunctionBasicData, crate_def_map::{ModuleData, ModuleId}}, krate::crate_manager::{CrateManager, CrateID}};
use crate::krate::crate_unit::{ModID};

use crate::ast::{Statement, Type, PrivateStatement, BlockStatement, Ident, ConstStatement, ConstrainStatement, LetStatement};
use crate::ast::{NoirPath, FunctionDefinition};
use crate::parser::Program;
use crate::NoirFunction;

use super::scope::{Scope as GenericScope, ScopeTree as GenericScopeTree, ScopeForest as GenericScopeForest};

type Scope = GenericScope<Ident, Type>;
type ScopeTree = GenericScopeTree<Ident, Type>;
type ScopeForest = GenericScopeForest<Ident, Type>;

use super::errors::{AnalyserError, Span};

mod expression;

pub struct TypeChecker<'a> {
    file_id : usize,
    module_data : &'a ModuleData,

    context: &'a Context,
    module_id : ModuleId,
    local_types : ScopeForest,
    errors : Vec<AnalyserError>,
}

impl<'a> TypeChecker<'a> {

    // fn new(file_id : usize, current_module : ModID,current_crate : CrateID, crate_manager : &CrateManager<Program>) -> TypeChecker {
    //     TypeChecker {file_id, current_module, current_crate, crate_manager, local_types : ScopeForest::new(), errors:Vec::new(), resolved_imports: HashMap::new()}
    // }
    
    fn find_function(&self, path : &Path) -> Option<&FunctionBasicData> {
        let func_id = super::resolve_call_path(self.context, self.module_id, path)?;
        Some(self.context.function_basic_data(func_id))
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

    pub fn lookup_local_identifier(&mut self, name : &Ident) -> Type {
        let scope_tree = self.local_types.current_scope_tree();
        scope_tree.find_key(name).expect("Compiler Error: Cannot find type for specified name. This should be caught by the Resolver pass").clone()
    }

    // pub fn check(ast : &mut Program, mod_id : ModID, crate_id : CrateID, crate_manager : &'a CrateManager<Program>) -> Result<(), Vec<AnalyserError>> {

    //     let mut type_checker = TypeChecker::new(ast.file_id, mod_id,crate_id, crate_manager);

    //     // Copy resolved imports
    //     type_checker.resolved_imports = ast.resolved_imports.clone();

    //      type_checker.type_check_ast(ast);

    //     if type_checker.errors.len() > 0 {
    //         return Err(type_checker.errors)
    //     }

    //     Ok(())
    // }

    fn type_check_ast(&mut self, ast : &mut Program) {
        for func in ast.functions.iter_mut() {
            self.type_check_func_def(func.def_mut());
        }
    }

    // Check that all assignments have the correct types
fn type_check_func_def(&mut self, func : &mut FunctionDefinition) {
    
    self.local_types.start_function();

    // Add function parameters to local types in the type checker
    for param in func.parameters.iter() {
        self.add_variable_declaration(param.0.clone(), param.1.clone());
    }

    let last_return_type = match self.type_check_block_stmt(&mut func.body) {
        Ok(lrt) => lrt,
        Err(_) => return // If an error is encountered, this will be picked up by the Reporter, lets not pollute stderr with mismatched return type
    };


    let declared_return_type = &func.return_type;

    let is_low_level = match &func.attribute{
        None => false,
        Some(attr) => attr.is_low_level()
    };

    if (&last_return_type != declared_return_type) & !is_low_level {
        let message = format!("mismatched types: Expected the function named `{}` to return the type `{}`, but got `{}`", &func.name.0.contents, declared_return_type, last_return_type);
        let err = AnalyserError::from_ident(self.file_id,message, &func.name);
        self.errors.push(err);
    }

    self.local_types.end_function();
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
        };
        // We immediately abort for an error, because we may not have the type information for the identifier 
        // in the statement which threw the error. We could make it more complex in the future, by continuing
        // when the type information is explicitly supplied, and or when it can be inferred from the statement
        match stmt_return_type {
            Ok(rt) => last_return_type = rt,
            Err(err) => {
                self.push_err(err);
                break
            }
        }   
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
        return Err(AnalyserError::type_mismatch(self.file_id,lhs_type, &expr_type, stmt.expression.span))
    }
    
    // Check if this type can be used in a Private statement
    if !lhs_type.can_be_used_in_priv() {
        let message = format!("the type {} cannot be used in a Private Statement", lhs_type);
        return Err(AnalyserError::from_expression(self.file_id,message, &stmt.expression));
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
        return Err(AnalyserError::type_mismatch(self.file_id,lhs_type, &expr_type, stmt.expression.span))
    }
    
    // Check if this type can be used in a Let statement
    if !lhs_type.can_be_used_in_let() {
        let message = format!("the type {} cannot be used in a Let Statement", lhs_type);
        return Err(AnalyserError::from_expression(self.file_id,message, &stmt.expression));
    }
    
    stmt.r#type = expr_type;
    
    // Update the Type checker to include this new identifier
    self.add_variable_declaration(stmt.identifier.clone(),stmt.r#type.clone());

    Ok(Type::Unit)
    
}
fn type_check_const_stmt(&mut self,stmt : &mut ConstStatement) -> Result<Type, AnalyserError>{
    let lhs_type = &stmt.r#type;

    if !(lhs_type == &Type::Constant || lhs_type == &Type::Unspecified) {
        let message = format!("constant statements can only contain constant types, found type {}", lhs_type);
        return Err(AnalyserError::from_expression(self.file_id,message, &stmt.expression));
    }
    let expr_type = self.type_check_expr(&mut stmt.expression)?;
    
    // Constant statements can only contain the Constant type
    if expr_type != Type::Constant {
        let message = format!("rhs of constrain statement must be of type `Constant`");
        return Err(AnalyserError::from_expression(self.file_id,message, &stmt.expression));
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
        let message = format!("only comparison operators can be used in a constrain statement");
        return Err(AnalyserError::Unstructured{file_id : self.file_id, message, span :stmt.0.operator.span()});
    };
    
    if !lhs_type.can_be_used_in_constrain() {
        let message = format!("found type {} . This type cannot be used in a constrain statement", lhs_type);
        return Err(AnalyserError::from_expression(self.file_id,message, &stmt.0.lhs));
    }
    if !rhs_type.can_be_used_in_constrain() {
        let message = format!("found type {} . This type cannot be used in a constrain statement", rhs_type);
        return Err(AnalyserError::from_expression(self.file_id,message, &stmt.0.rhs));
    }

    Ok(Type::Unit)    // XXX: We leave upper bound checks until runtime, but it is certainly possible to add them to the analyser
}
}



