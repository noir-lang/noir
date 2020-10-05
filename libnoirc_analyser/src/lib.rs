mod resolve;

mod type_check;
use type_check::TypeChecker;

use resolve::Resolver;
use libnoirc_ast::FunctionDefinition;
/// This module is for now just a placeholder
/// We want the analyser to do quite a few things such as:
/// - Be able to check for unused variables (Resolver)
/// - Check if function parameters and arguments are lined up
/// - Check if all variables are in scope (Resolver)
/// - Check if any integer types are too big. We can have field element return the max field size
/// - Should modify the AST for the lazy operations. priv k = a + 5 will insert (a+5) into all places where k is needed unless it is a mul by arith gate
///  This means the compiler only needs to constrain private statements when they see them. I think it also means we can refactor env, it will only be used for scope management + symbol table
/// - Fill in inferred types for witnesses priv k = x as u8, should modify k to be the u8 Type
/// - Check array boundaries, check the array is being indexed with a constant or a u128, if field element, check boundaries (this is also checked at runtime, it might make sense to leave it there)
///
///
use libnoirc_ast::{Statement, ImportStatement};
use libnoirc_ast::{SymbolTable, NoirFunction};
use libnoirc_parser::Program;

use std::collections::HashMap;

pub struct CheckedProgram{
    pub imports: Vec<ImportStatement>,
    pub statements: Vec<Statement>,
    pub functions: HashMap<String, (NoirFunction, SymbolTable)>,
    pub main: Option<FunctionDefinition>,
}

pub fn check(ast: Program) -> (Program, SymbolTable) {

    // Resolver
    let ast = Resolver::resolve(ast);
    
    // Type checker
    let ast = TypeChecker::check(ast);

    let symbol_table = build_symbol_table(&ast);

    (ast, symbol_table)
}

fn build_symbol_table(ast: &Program) -> SymbolTable {
    let mut root_table = SymbolTable::new();

    // Add the low level standard library symbol table
    // XXX: We will also compile and add the high level library here, but we do not have any high level std library constructs yet
    // Once modules are implemented, this(stdlib) will then move upto the module layer
    load_low_level_libraries_into_symbol_table(&mut root_table);

    load_local_functions_into_symbol_table(ast, &mut root_table);

    root_table
}

fn load_local_functions_into_symbol_table(ast: &Program, table: &mut SymbolTable) {
    for func_def in ast.functions.iter() {
        table.insert_func_def(func_def);
    }

    // Add main function separately as it is not inside of the list of functions.
    // XXX: Should we just add it like any other function and look it up with the symbol table?
    match &ast.main {
        Some(main_func) => table.insert_func_def(main_func),
        None => {}
    };
}

fn load_low_level_libraries_into_symbol_table(table: &mut SymbolTable) {
    low_level_std_lib::populate_symbol_table(table);
}