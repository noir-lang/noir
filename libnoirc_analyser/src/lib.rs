mod private_statement;
use libnoirc_ast::{CallExpression, FunctionDefinition, FunctionLiteral as Function, Ident, NoirPath};
/// This module is for now just a placeholder
/// We want the analyser to do quite a few things such as:
/// - Be able to check for unused variables
/// - Check if function parameters and arguments are lined up
/// - Check if all variables are in scope
/// - Check if any integer types are too big. We can have field element return the max field size
/// - Should modify the AST for the lazy operations. priv k = a + 5 will insert (a+5) into all places where k is needed unless it is a mul by arith gate
///  This means the compiler only needs to constrain private statements when they see them. I think it also means we can refactor env, it will only be used for scope management + symbol table
/// - Fill in inferred types for witnesses priv k = x as u8, should modify k to be the u8 Type
/// - Check array boundaries, check the array is being indexed with a constant or a u128, if field element, check boundaries
///
///
/// Note: Given private x = arith
/// Throw an error, if private x is never used.
/// This allows us to constrain x straight away
use libnoirc_ast::{Expression, Statement, Type};
use libnoirc_ast::{Scope, SymbolInformation, SymbolTable};
use libnoirc_parser::Program;

pub fn check(ast: Program) -> Program {
    let symbol_table = build_symbol_table(&ast);
    argument_size_check(&ast, &symbol_table);
    ast
}

pub fn build_symbol_table(ast: &Program) -> SymbolTable {
    let mut root_table = SymbolTable::new();

    // Add the low level standard library symbol table
    // We will also compile and add the high level library here, but we do not have any high level std library constructs yet
    low_level_std_lib::populate_symbol_table(&mut root_table);

    build_symbol_table_for_function(ast, Scope::global(), &mut root_table);
    build_symbol_table_for_witnesses(ast, Scope::global(), &mut root_table);

    root_table
}

/// Witnesses are declared using the `priv` keyword and are contained in private statements
fn build_symbol_table_for_function(ast: &Program, scope: Scope, table: &mut SymbolTable) {
    for func_def in ast.functions.iter() {
        insert_func_def(table, func_def)
    }

    // Add main function separately as it is not inside of the list of functions.
    // XXX: Should we just add it like any other function and look it up with the symbol table?
    match &ast.main {
        Some(main_func) => insert_func_def(table, main_func),
        None => {}
    };
}
fn insert_func_def(table: &mut SymbolTable, func_def: &FunctionDefinition) {
    let (func_name, func) = parse_function_declaration(func_def);
    let function_already_declared =  table.look_up(&func_name).is_some(); 
    if function_already_declared {
        panic!("Another symbol has been defined with the name {}", &func_name.0)
    }
    table.insert(func_name, SymbolInformation::Function(func));
}
/// Witnesses are declared using the `priv` keyword and are contained in private statements
fn build_symbol_table_for_witnesses(ast: &Program, scope: Scope, table: &mut SymbolTable) {
    for node in ast.statements.iter() {
        match node {
            Statement::Private(private) => {
                let resolved_type = type_resolution(&private.r#type, &private.expression);
                table.insert(
                    private.identifier.clone(),
                    SymbolInformation::Variable(resolved_type, scope),
                );
            }
            _ => {}
        }
    }
}

fn get_type_from_expression(expr: &Expression) -> Type {
    todo!()
}

// First resolve the RHS type
// Then if the lhs is unknown, then we assign it the type we inferred from the rhs
fn type_resolution(lhs_type: &Type, rhs: &Expression) -> Type {
    let rhs_type = get_type_from_expression(rhs);

    let inferred_type = rhs_type;
    if lhs_type != &Type::Unknown {
        assert_eq!(lhs_type, &inferred_type);
    }
    inferred_type
}

/// Convert a function declarations into a function object
fn parse_function_declaration(func_dec: &FunctionDefinition) -> (Ident, Function) {
    let func_name = func_dec.name.clone();
    let body = func_dec.func.body.clone();
    let parameters = func_dec.func.parameters.clone();

    // Store function in evaluator
    (func_name, Function { body, parameters })
}

// Checks if each function has the right argument call size
fn argument_size_check(ast: &Program, symbol_table : &SymbolTable) {
    let main_func = match &ast.main {
        Some(func) => func,
        None => panic!("No main function found"),
    };

    for statement in main_func.func.body.0.iter() {
        match statement {
            Statement::Expression(expr) => expression_check(ast, symbol_table,&expr.0),
            Statement::Private(private_stmt) => {
                expression_check(ast, symbol_table, &private_stmt.expression);
            }
            Statement::Constrain(constrain_stmt) => {
                expression_check(ast, symbol_table, &constrain_stmt.0.lhs);
                expression_check(ast, symbol_table,&constrain_stmt.0.rhs);
            }
            k => {
                dbg!(k);
            }
        }
    }
}

fn expression_check(ast: &Program, symbol_table : &SymbolTable, expr: &Expression) {
    match expr {
        Expression::Call(noir_path, call_expr) => {
            // First check that the function we are calling, has been declared before
            check_call_expression(ast, symbol_table,  noir_path, call_expr);
        },
        Expression::Infix(infix_expr) => {
            expression_check(ast,symbol_table, &infix_expr.lhs);
            expression_check(ast, symbol_table,&infix_expr.rhs);
        }
        _ => {}
    }
}

fn check_call_expression(ast: &Program, symbol_table : &SymbolTable, noir_path : &NoirPath, call_expr: &CallExpression) {
    let valid = symbol_table.valid_func(noir_path.clone(), &call_expr.func_name);

    // Check if we have the correct number of arguments
    // check_correct_number_of_arguments(func, call_expr.arguments.len());
}

fn find_function<'a>(ast: &'a Program, func_name: &str) -> Option<&'a FunctionDefinition> {
    for function in ast.functions.iter() {
        if &function.name.0 == func_name {
            return Some(function);
        }
    }

    return None;
}

fn check_correct_number_of_arguments(func_def: &FunctionDefinition, num_called: usize) -> bool {
    let num_parameters = func_def.func.parameters.len();
    match num_parameters == num_called {
        true => return true,
        false => panic!(
            "The function {} takes {} parameters, but {} were supplied",
            func_def.name.0, num_parameters, num_called
        ),
    }
}
