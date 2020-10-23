use crate::ast::{Expression, Statement, NoirPath, FunctionDefinition};
use crate::{SymbolTable, NoirFunction};
use crate::ast::{Ident, BlockStatement, PrivateStatement, ConstrainStatement, ConstStatement, LetStatement};
use crate::parser::Program;
use super::scope::{Scope as GenericScope, ScopeTree as GenericScopeTree, ScopeForest as GenericScopeForest};

/// Checks that each variable was declared before it's use
/// Checks that there are no unused variables

type Scope = GenericScope<Ident, usize>;
type ScopeTree = GenericScopeTree<Ident, usize>;
type ScopeForest = GenericScopeForest<Ident, usize>;

mod expression;

pub struct Resolver<'a>{
    table : &'a SymbolTable,
    local_declarations : ScopeForest
}

impl<'a> Resolver<'a> {

    fn from_symbol_table(table : &'a SymbolTable) -> Resolver<'a> {
        Resolver {table, local_declarations : ScopeForest::new()}
    }

    fn add_variable_decl(&mut self, name : Ident) {

        let scope = self.local_declarations.get_mut_scope();
        let is_new_entry = scope.add_key_value(name.clone(),0);

        if !is_new_entry {
            panic!("\nMultiple variables cannot have the same name, duplicate declarations of {:?}\n", &name.0)
        }
        
    }
   
    // Checks for a variable and increments a counter
    // to signify that it has been fetched
    fn find_variable(&mut self, name : &Ident) -> bool {
               
        let scope_tree = self.local_declarations.current_scope_tree();
        let variable = scope_tree.find_key(name);
        
        if let Some(variable_found) = variable {
            *variable_found = *variable_found + 1;
            return true
        } 
        return false
    }

    fn find_function(&self, path : &NoirPath, func_name : &Ident) -> Option<NoirFunction> {   
        self.table.look_up_func(path.clone(), &func_name)
    }

    // Checks if all variables have been correctly scoped
    // XXX: Can check here that main() has no return type
    // We can probably check for duplicate var and func names in here
    pub fn resolve(mut ast : Program, table : &SymbolTable) -> Program {

        // Add functions into this, so that call expressions can be resolved
        let mut resolver = Resolver::from_symbol_table(table);

        ast.main = match ast.main.clone() {
            Some(main_func) =>    {    
                Some(resolver.resolve_func_def(main_func))
            },
            None => None
        };

        ast = resolver.resolve_ast(ast);

        ast.modules = ast.modules.into_iter().map(|(module_id, module)| {
            (module_id, resolver.resolve_ast(module))
        }).collect();

        ast
    }

    fn resolve_ast(&mut self, mut ast : Program) -> Program {
        ast.functions = ast.functions.into_iter().map(|func| {
            self.resolve_func_def(func)
        }).collect();

        ast
    }

    fn check_for_unused_variables_in_scope_tree(scope_decls : &ScopeTree) {
        for scope in scope_decls.0.iter(){
            Resolver::check_for_unused_variables_in_local_scope(scope)
        }
    }

    fn check_for_unused_variables_in_local_scope(decl_map : &Scope) {
        let unused_variables = decl_map.predicate(|kv :&(&Ident, &usize)| -> bool {
            
            let variable_name = kv.0;
            let num_times_fetched = kv.1;
            
            let has_underscore_prefix = variable_name.0.starts_with("_");

            if *num_times_fetched == 0 && !has_underscore_prefix {
                return true
            }
            false
        });

        let variables_names : Vec<_>= unused_variables.map(|(var_name, _)|var_name ).collect();

        if variables_names.len() > 0 {
            panic!("Unused variables detected. The following variables are unused : {:?}", variables_names)
        } 
    }

    fn resolve_func_def(&mut self, mut func : FunctionDefinition) -> FunctionDefinition{

        // Add a new scope tree as we do not want the function to have access to the caller's scope
        self.local_declarations.start_function();
        
        // Add function parameters so they can be seen as declared
        for param in func.parameters.iter() {
            self.add_variable_decl(param.0.clone());
        }
        
        self.resolve_block_stmt(&mut func.body);
        
        let function_scope_tree = self.local_declarations.end_function();
        Resolver::check_for_unused_variables_in_scope_tree(&function_scope_tree);
        
        func
    }

    fn resolve_block_stmt(&mut self, block : &BlockStatement) {
  
        for stmt in block.0.iter() {
            match stmt {
                Statement::Private(priv_stmt) => {
                    self.resolve_private_stmt(priv_stmt);
                },
                Statement::Expression(expr) => {
                    if !self.resolve_expr(&expr.0) {
                        panic!("Could not resolve the expression private statement {:?}", expr.0);
                    };
                },
                Statement::Assign(assign_stmt) => {
                    let ident = &assign_stmt.0.identifier;
                    let rhs = &assign_stmt.0.rhs;

                    if !self.find_variable(ident) {
                        panic!("The variable {} is being used in an assignment statement, but has not been declared", ident.0)
                    }

                    if !self.resolve_expr(rhs) {
                        panic!("Could not resolve the rhs in the assign statement {:?}", rhs);
                    }

                }
                Statement::Block(_) => {
                    panic!("Currently we do not support block statements inside of block statements")
                },
                Statement::Const(const_stmt) => {
                    self.resolve_const_stmt(const_stmt);
                },
                Statement::If(_) => {
                    panic!("[Possible Deprecation] : If statements are not implemented yet, however they might be deprecated for if expressions");
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

    fn resolve_declaration_stmt(&mut self, identifier : &Ident, expr : &Expression, stmt_type : &str) {
        // Add the new variable to the list of declarations
        self.add_variable_decl(identifier.clone());

        // Check that the expression on the RHS is using variables which have already been declared
        if !self.resolve_expr(expr) {
            panic!("Could not resolve the expression in the {} statement {:?}", stmt_type,expr);
        };
    }
    fn resolve_private_stmt(&mut self, private_stmt : &PrivateStatement) {
        self.resolve_declaration_stmt(&private_stmt.identifier, &private_stmt.expression, "private");
    }
    fn resolve_const_stmt(&mut self, const_stmt : &ConstStatement) {
        self.resolve_declaration_stmt(&const_stmt.identifier, &const_stmt.expression, "constant");
    }
    fn resolve_let_stmt(&mut self, let_stmt : &LetStatement) {
        self.resolve_declaration_stmt(&let_stmt.identifier, &let_stmt.expression, "let");
    }
    fn resolve_constrain_stmt(&mut self, constrain_stmt : &ConstrainStatement) {
        self.resolve_infix_expr(&constrain_stmt.0, "constrain statement");
    }

}