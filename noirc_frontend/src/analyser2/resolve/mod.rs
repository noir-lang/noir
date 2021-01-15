

use fm::FileID;

use crate::{Path, ast::{Expression, Statement, FunctionDefinition}, hir::{Context, basic_data::{FunctionBasicData}, crate_def_map::{LocalModuleId, ModuleData, ModuleDefId, ModuleId}}};
use crate::ast::{Ident, BlockStatement, PrivateStatement, ConstrainStatement, ConstStatement, LetStatement};
use super::{errors::ResolverErrorKind, scope::{Scope as GenericScope, ScopeTree as GenericScopeTree, ScopeForest as GenericScopeForest}};

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

// Turn a module to a scope, which is just Map<Name -> ModuleDefId>
// The module scope is the global scope and functions have a local scope
// The scope of for loops is such that it extends the outer scope
pub struct Resolver<'a>{
    file_id : usize,
    context : &'a Context,
    local_declarations : ScopeForest,
    module_data : &'a ModuleData,
    module_id : ModuleId,
    errors : Vec<AnalyserError>
}

impl<'a> Resolver<'a> {

    fn new(context : &'a Context, module_id : ModuleId, module_data : &'a ModuleData) -> Resolver<'a> {
        let file_id : FileID = module_data.origin.into();
        Resolver {file_id : file_id.as_usize(), module_id, module_data, context, local_declarations : ScopeForest::new(), errors: Vec::new()}
    }

    pub fn resolve_crates(mut context : Context) -> Result<Context,Vec<AnalyserError>> {

        // Loop over each crate we have
        for crate_id in context.crates() {
            // Get crate
            let krate = context.def_map(crate_id).expect("expected a CrateDefMap. This is an ice.");

            for (index, module) in krate.modules() {
                let module_id = ModuleId{krate : crate_id, local_id : LocalModuleId(index)};
                let mut resolver = Resolver::new(&context,module_id, module);
                resolver.resolve_module(&context, module);

            }


        }


        Ok(context)
    }
    
    fn resolve_module(&mut self, context : &Context, module : &ModuleData) {
        // Take all of the definitions in the module
        // We have two types of definitions at the moment: Modules and Functions
        let definitions = module.scope.definitions();

        for def in definitions {
            match def {
                ModuleDefId::ModuleId(mod_id) => {
                    // We can ignore this, as we are iterating over all of the modules
                    // alternatively, we could start from the root and Resolve modules
                    
                    todo!()
                },
                ModuleDefId::FunctionId(func_id) => {
                    todo!()
                }
            }
        }

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

    fn find_function(&self, path : &Path) -> Option<&FunctionBasicData> {
        let func_id = super::resolve_call_path(self.context, self.module_id, path)?;
        Some(self.context.function_basic_data(func_id))
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
        self.resolve_infix_expr(&constrain_stmt.0);
    }
}
