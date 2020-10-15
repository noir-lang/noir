use noirc_frontend::ast::{Statement, Type, PrivateStatement, BlockStatement, Ident, ConstStatement, ConstrainStatement, LetStatement};
use noirc_frontend::ast::{NoirPath, FunctionDefinition};
use noirc_frontend::parser::Program;
use noirc_frontend::{SymbolTable, NoirFunction};
use std::collections::HashMap;

mod expression;

pub struct TypeChecker<'a> {
    table : &'a SymbolTable,
    local_types : HashMap<Ident, Type>
}

impl<'a> TypeChecker<'a> {

    fn from_symbol_table(table : &SymbolTable) -> TypeChecker {
        TypeChecker {table, local_types : HashMap::new()}
    }

    pub fn clear(&mut self) {
        self.local_types.clear();
    }

    pub fn add_variable_declaration(&mut self, name : Ident, typ : Type) {
        let is_duplicate = self.local_types.insert(name, typ).is_some();

        if is_duplicate {
            panic!("Two parameters in a function cannot have the same name")
        }
    }

    fn find_function(&self, path : &NoirPath, func_name : &Ident) -> Option<NoirFunction> {   
        self.table.look_up_func(path.clone(), &func_name)
    }
    pub fn lookup_local_identifier(&self, name : &Ident) -> Type {
        self.local_types.get(&(name.clone())).unwrap().clone()
    }

    pub fn check(mut ast : Program, table : &SymbolTable) -> Program {

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

        ast
    }

    fn type_check_ast(&mut self, mut ast : Program) -> Program {

        ast.functions = ast.functions.into_iter().map(|func| {
            self.type_check_func_def(func)
        }).collect();

        ast
    }

    // Check that all assignments have the correct types
fn type_check_func_def(&mut self, mut func : FunctionDefinition) -> FunctionDefinition {

    // Add function parameters to local types in the type checker
    for param in func.parameters.iter() {
        self.add_variable_declaration(param.0.clone(), param.1.clone());
    }

    let last_return_type = self.type_check_block_stmt(&mut func.body);

    let declared_return_type = &func.return_type;

    let is_foreign = match &func.attribute{
        None => false,
        Some(attr) => attr.is_foreign()
    };

    if (&last_return_type != declared_return_type) & !is_foreign {
        panic!("mismatched types: Expected the function named `{}` to return the type `{}`, but got `{}`", &func.name.0, declared_return_type, last_return_type);
    }

    self.clear(); 

    func
}

// Check that all assignments have the correct types
fn type_check_block_stmt(&mut self, block : &mut BlockStatement) -> Type {

    let mut last_return_type = Type::Unit;

    for stmt in block.0.iter_mut() {
        match stmt {
            Statement::Private(ref mut priv_stmt) => {
                self.type_check_private_stmt(priv_stmt);
                last_return_type = Type::Unit;
            },
            Statement::Expression(expr) => {
                last_return_type = self.type_check_expr(&mut expr.0);
            },
            Statement::Block(_) => {
                panic!("Currently we do not support block statements inside of block statements")
            },
            Statement::Const(const_stmt) => {
                self.type_check_const_stmt(const_stmt);
                last_return_type = Type::Unit;
            },
            Statement::If(_) => {
                last_return_type = Type::Unit;
                panic!("[Possible Deprecation] : If statements are not implemented yet, however they might be deprecated for if expressions");
            },
            Statement::Let(let_stmt) => {
                self.type_check_let_stmt(let_stmt);
                last_return_type = Type::Unit;
            },
            Statement::Constrain(constr_stmt) =>{
                self.type_check_constrain_stmt(constr_stmt);
                last_return_type = Type::Unit;
            },
            Statement::Public(_) => {
                last_return_type = Type::Unit;
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

    last_return_type
}

fn type_check_private_stmt(&mut self,stmt : &mut PrivateStatement) {
    let mut lhs_type = &stmt.r#type; 
    let expr_type = self.type_check_expr(&mut stmt.expression);

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
        panic!("\n\nType mismatch: Expected: {:?} got: {:?} \n\n", lhs_type, expr_type)
    }

    // Check if this type can be used in a Private statement
    if !lhs_type.can_be_used_in_priv() {
        panic!("The Type {:?} cannot be used in a Private Statement", lhs_type)
    }

    stmt.r#type = expr_type;

    // Update the Type checker to include this new identifier
    self.add_variable_declaration(stmt.identifier.clone(),stmt.r#type.clone());
  
}
fn type_check_let_stmt(&mut self,stmt : &mut LetStatement) {
    let mut lhs_type = &stmt.r#type; 
    let expr_type = self.type_check_expr(&mut stmt.expression);

    // Witness types cannot be used in a let statement, they are for Private statements

    // First check if the LHS is unspecified
    // If so, then we give it the same type as the RHS
    if lhs_type == &Type::Unspecified {
        lhs_type = &expr_type;
    }

    // Now check if LHS is the same type as the RHS
    // Importantly, we do not co-erce any types implicitly
    if lhs_type != &expr_type {
        panic!("\n\nType mismatch: Expected: {:?} got: {:?} \n\n", lhs_type, expr_type)
    }

    // Check if this type can be used in a Let statement
    if !lhs_type.can_be_used_in_let() {
        panic!("The Type {:?} cannot be used in a Let Statement", lhs_type)
    }
  
    stmt.r#type = expr_type;

    // Update the Type checker to include this new identifier
    self.add_variable_declaration(stmt.identifier.clone(),stmt.r#type.clone());

}
fn type_check_const_stmt(&mut self,stmt : &mut ConstStatement) {
    let lhs_type = &stmt.r#type;
    if !(lhs_type == &Type::Constant || lhs_type == &Type::Unspecified) {
        panic!("Constant statements can only contain constant types")
    }
    let expr_type = self.type_check_expr(&mut stmt.expression);

    // Constant statements can only contain the Constant type
    if expr_type != Type::Constant {
        panic!("RHS of constrain statement must be of type `Constant`");
    }

    stmt.r#type = expr_type;

    // Update the Type checker to include this new identifier
    self.add_variable_declaration(stmt.identifier.clone(),stmt.r#type.clone());
}
fn type_check_constrain_stmt(&mut self,stmt : &mut ConstrainStatement) {
    let lhs_type = self.type_check_expr(&mut stmt.0.lhs);
    let rhs_type = self.type_check_expr(&mut stmt.0.rhs);

    // Are there any restrictions on the operator for constrain statements
    if !stmt.0.operator.is_comparator() {
        panic!("Only comparison operators can be used in a constrain statement")
    };
    
    if !lhs_type.can_be_used_in_constrain() {
        panic!("LHS is of type {:?} . This type cannot be used in a constrain statement", lhs_type)
    }
    if !rhs_type.can_be_used_in_constrain() {
        panic!("RHS is of type {:?} . This type cannot be used in a constrain statement", rhs_type)
    }

    // XXX: We leave upper bound checks until runtime, but it is certainly possible to add them to the analyser
}
}



