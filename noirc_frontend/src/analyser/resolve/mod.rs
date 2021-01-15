use std::collections::HashMap;

use crate::{ast::{Expression, Statement, NoirPath, FunctionDefinition}};
use crate::NoirFunction;
use crate::ast::{Ident, BlockStatement, PrivateStatement, ConstrainStatement, ConstStatement, LetStatement};
use crate::parser::Program;
use super::{errors::ResolverErrorKind, scope::{Scope as GenericScope, ScopeTree as GenericScopeTree, ScopeForest as GenericScopeForest}};
use crate::krate::crate_manager::{CrateManager, CrateID};
use crate::krate::crate_unit::{ModID};

/// Checks that each variable was declared before it's use
/// Checks that there are no unused variables
struct ResolverMeta{
    num_times_used : usize,
    span : Span,
}

type Scope = GenericScope<String, ResolverMeta>;
type ScopeTree = GenericScopeTree<String, ResolverMeta>;
type ScopeForest = GenericScopeForest<String, ResolverMeta>;

mod expression;
use super::errors::{AnalyserError, Span};

pub struct Resolver<'a>{
    file_id : usize,
    crate_manager : &'a CrateManager<Program>,
    current_module : ModID,
    current_crate : CrateID, // XXX: We should encode this into the module_id
    local_declarations : ScopeForest,
    resolved_imports : HashMap<String, (ModID, CrateID)>,
    errors : Vec<AnalyserError>
}

impl<'a> Resolver<'a> {

    fn new( file_id : usize, current_module : ModID,current_crate : CrateID, crate_manager : &'a CrateManager<Program>) -> Resolver<'a> {
        Resolver {file_id, local_declarations : ScopeForest::new(), current_module, current_crate, errors: Vec::new(),resolved_imports: HashMap::new(),crate_manager}
    }

    fn add_variable_decl(&mut self, name : Ident) {

        let scope = self.local_declarations.get_mut_scope();
        let resolver_meta = ResolverMeta {num_times_used : 0, span : name.0.span()};
        let is_new_entry = scope.add_key_value(name.0.contents.clone(), resolver_meta);

        if !is_new_entry {
            let first_decl = scope.find(&name.0.contents).unwrap();
            let err = ResolverErrorKind::DuplicateDefinition{first_span: first_decl.span, second_span : name.0.span(), ident: name.0.contents}.into_err(self.file_id);
            self.push_err(err);
        }

    }

    fn push_err(&mut self, err : impl Into<AnalyserError>) {
        self.errors.push(err.into())
    }
   
    // Checks for a variable and increments a counter
    // to signify that it has been fetched
    fn find_variable(&mut self, name : &Ident) -> bool {
               
        let scope_tree = self.local_declarations.current_scope_tree();
        let variable = scope_tree.find_key(&name.0.contents);
        
        if let Some(variable_found) = variable {
            variable_found.num_times_used = variable_found.num_times_used + 1;
            return true
        } 
        return false
    }

    fn find_function(&self, path : &NoirPath, func_name : &Ident) -> Option<&NoirFunction> {
        let module = self.resolve_call_path(self.current_crate, self.current_module, path)?;
        Some(module.find_function(&func_name.0.contents)?.into())
    }

    // Resolve `foo::bar` in foo::bar::call() to the module with the function
    // This function has been duplicated, due to the fact that we cannot make it generic over the Key
    pub fn resolve_call_path(&self, current_crate : CrateID, current_module : ModID, path : &NoirPath) -> Option<&'_ Program> {
        match path {
            NoirPath::Current => {
                let krate = self.crate_manager.get_crate_with_id(current_crate)?;
                krate.get_module(current_module)
            },
            NoirPath::External(pth) => {
                let path = pth.first()?.clone();
                let (mod_id, crate_id ) = self.resolved_imports.get(&path.0.contents)?;
                let krate = self.crate_manager.get_crate_with_id(*crate_id)?;
                krate.get_module(*mod_id)
            }
        }
        }

    fn resolve_imports(&mut self, ast : &mut Program) {
        for import in ast.imports.iter() {
            let (key, mod_id, crate_id) = super::resolve_import(import, self.crate_manager);
            ast.resolved_imports.insert(key.0.contents, (mod_id, crate_id));
        }
        // Copy imports to resolver
        self.resolved_imports = ast.resolved_imports.clone();
    }

    // Checks if all variables have been correctly scoped
    // XXX: Can check here that main() has no return type
    // We can probably check for duplicate var and func names in here
    pub fn resolve(ast : &mut Program, mod_id : ModID, crate_id : CrateID, crate_manager : &'a CrateManager<Program>) -> Result<(),Vec<AnalyserError>> {

        // Add functions into this, so that call expressions can be resolved
        let mut resolver = Resolver::new(ast.file_id, mod_id, crate_id, crate_manager);

        // Resolve Import paths and copy to Resolver
        resolver.resolve_imports(ast);

        // Resolve AST
        for func in ast.functions.iter(){
            resolver.resolve_func_def(func.def());
        }

        if resolver.errors.len() > 0 {
            Err(resolver.errors)
        } else {
            Ok(())
        }
    }

    fn check_for_unused_variables_in_scope_tree(&mut self, scope_decls : &ScopeTree) {
        for scope in scope_decls.0.iter(){
            self.check_for_unused_variables_in_local_scope(scope)
        }
    }

    fn check_for_unused_variables_in_local_scope(&mut self, decl_map : &Scope) {
        let unused_variables = decl_map.predicate(|kv :&(&String, &ResolverMeta)| -> bool {
            
            let variable_name = kv.0;
            let metadata = kv.1;
            
            let has_underscore_prefix = variable_name.starts_with("_"); // XXX: This is used for development mode, and will be removed

            if metadata.num_times_used == 0 && !has_underscore_prefix {
                return true
            }
            false
        });

        for (unused_var, meta) in unused_variables.into_iter() {
            let span = meta.span;
            let ident = unused_var.clone();
            let err = ResolverErrorKind::UnusedVariables{span, ident}.into_err(self.file_id);
            self.push_err(err);
        }
    }

    fn resolve_func_def(&mut self, func : &FunctionDefinition) {

        // Add a new scope tree as we do not want the function to have access to the caller's scope
        self.local_declarations.start_function();
        
        // Add function parameters so they can be seen as declared
        for param in func.parameters.iter() {
            self.add_variable_decl(param.0.clone());
        }
        
        self.resolve_block_stmt(&func.body);
        
        let function_scope_tree = self.local_declarations.end_function();
        self.check_for_unused_variables_in_scope_tree(&function_scope_tree);
        
    }

    fn resolve_block_stmt(&mut self, block : &BlockStatement) {
  
        for stmt in block.0.iter() {
            match stmt {
                Statement::Private(priv_stmt) => {
                    self.resolve_private_stmt(priv_stmt);
                },
                Statement::Expression(expr) => {
                    if !self.resolve_expr(&expr) {
                        let message = format!("Could not resolve the expression");
                        let err = AnalyserError::from_expression(self.file_id,message, expr);
                        self.push_err(err);
                    };
                },
                Statement::Block(_) => {
                    panic!("Currently we do not support block statements inside of block statements")
                },
                Statement::Const(const_stmt) => {
                    self.resolve_const_stmt(const_stmt);
                },
                Statement::Let(let_stmt) => {
                    self.resolve_let_stmt(let_stmt);
                },
                Statement::Constrain(constr_stmt) =>{
                    self.resolve_constrain_stmt(constr_stmt);
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
            }
        }
    }

    fn resolve_declaration_stmt(&mut self, identifier : &Ident, expr : &Expression) {
        // Add the new variable to the list of declarations
        self.add_variable_decl(identifier.clone());

        // Check that the expression on the RHS is using variables which have already been declared
        self.resolve_expr(expr);
    }
    fn resolve_private_stmt(&mut self, private_stmt : &PrivateStatement) {
        self.resolve_declaration_stmt(&private_stmt.identifier, &private_stmt.expression);
    }
    fn resolve_const_stmt(&mut self, const_stmt : &ConstStatement) {
        self.resolve_declaration_stmt(&const_stmt.identifier, &const_stmt.expression);
    }
    fn resolve_let_stmt(&mut self, let_stmt : &LetStatement) {
        self.resolve_declaration_stmt(&let_stmt.identifier, &let_stmt.expression);
    }
    fn resolve_constrain_stmt(&mut self, constrain_stmt : &ConstrainStatement) {
        
        
        todo!()
        // self.resolve_infix_expr(&constrain_stmt.0);
    }

}