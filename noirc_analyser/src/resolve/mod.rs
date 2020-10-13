use noirc_frontend::ast::{Expression, Statement, NoirPath, FunctionDefinition};
use noirc_frontend::{SymbolTable, NoirFunction};
use noirc_frontend::ast::{Ident, BlockStatement, PrivateStatement, ConstrainStatement, ConstStatement, LetStatement};
use noirc_frontend::parser::Program;


/// Checks that each variable was declared before it's use
/// Checks that there are no unused variables

mod expression;

use std::collections::{HashMap};

pub struct Resolver<'a>{
    table : &'a SymbolTable,
    local_declarations : HashMap<Ident, usize>
}


impl<'a> Resolver<'a> {

    fn from_symbol_table(table : &'a SymbolTable) -> Resolver<'a> {
        Resolver {table, local_declarations : HashMap::new()}
    }

    fn add_variable_decl(&mut self, name : Ident) {
        let is_new_entry = self.local_declarations.insert(name.clone(),0).is_none();

        if !is_new_entry {
            panic!("\nMultiple variables cannot have the same name, duplicate declarations of {:?}\n", &name.0)
        }
    }

    // Checks for a variable and increments a counter
    // to signify that it has been fetched
    fn find_variable(&mut self, name : &Ident) -> bool {
        match self.local_declarations.get_mut(name) {
            Some(value) => {
                *value = *value + 1;
                return true
            },
            None => return false
        }
    }
    fn find_function(&self, path : &NoirPath, func_name : &Ident) -> Option<NoirFunction> {   
        self.table.look_up_func(path.clone(), &func_name)
    }

    fn clear(&mut self) {
        self.local_declarations.clear();
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

    fn resolve_func_def(&mut self, mut func : FunctionDefinition) -> FunctionDefinition{
        // Add function parameters so they can be seen as declared
        for param in func.literal.parameters.iter() {
            self.add_variable_decl(param.0.clone());
        }

        self.resolve_block_stmt(&mut func.literal.body);

        let mut unused_variables = Vec::new();
        for (variable_name, value) in self.local_declarations.iter(){
            let has_underscore_prefix = variable_name.0.starts_with("_");
            if *value == 0 && !has_underscore_prefix{
                unused_variables.push(variable_name)
            }
        }

        if unused_variables.len() > 0 {
            panic!("Unused variables detected. The following variables are unused : {:?}", unused_variables)
        } 

        self.clear(); 

        func
    }

    fn resolve_block_stmt(&mut self, block : &mut BlockStatement) {
  
        for stmt in block.0.iter_mut() {
            match stmt {
                Statement::Private(ref mut priv_stmt) => {
                    self.resolve_private_stmt(priv_stmt);
                },
                Statement::Expression(expr) => {
                    if !self.resolve_expr(&expr.0) {
                        panic!("Could not resolve the expression private statement {:?}", expr.0);
                    };
                },
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
    fn resolve_private_stmt(&mut self, private_stmt : &mut PrivateStatement) {
        self.resolve_declaration_stmt(&private_stmt.identifier, &private_stmt.expression, "private");
    }
    fn resolve_const_stmt(&mut self, const_stmt : &mut ConstStatement) {
        self.resolve_declaration_stmt(&const_stmt.identifier, &const_stmt.expression, "constant");
    }
    fn resolve_let_stmt(&mut self, let_stmt : &mut LetStatement) {
        self.resolve_declaration_stmt(&let_stmt.identifier, &let_stmt.expression, "let");
    }
    fn resolve_constrain_stmt(&mut self, constrain_stmt : &mut ConstrainStatement) {
        self.resolve_infix_expr(&constrain_stmt.0, "constrain statement");
    }

}