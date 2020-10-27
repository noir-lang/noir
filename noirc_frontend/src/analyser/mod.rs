
mod errors;
use errors::AnalyserError;

pub mod scope;

mod attribute_check;
use attribute_check::AttributeChecker;

mod resolve;
use resolve::Resolver;

mod type_check;
use type_check::TypeChecker;

use crate::ast::FunctionDefinition;
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
use crate::ast::{Statement, ImportStatement};
use crate::symbol_table::{SymbolTable, NoirFunction, SymbolInformation};
use crate::parser::Program;

use std::collections::HashMap;

pub struct CheckedProgram{
    pub imports: Vec<ImportStatement>,
    pub statements: Vec<Statement>,
    pub functions: HashMap<String, (NoirFunction, SymbolTable)>,
    pub main: Option<FunctionDefinition>,
}

pub fn check(ast: Program) -> Result<(Program, SymbolTable), Vec<AnalyserError>> {
    check_program(ast, false)
}

// We need to bootstrap the standard library. This code will be removed once we stop interpreting the AST
// Or we move the stdlib symbol table to be on symbol table by default. (For this it would need to also be recompiled however)
fn check_program(ast : Program, is_std_lib : bool) -> Result<(Program, SymbolTable), Vec<AnalyserError>> {

    // Attribute checker
    AttributeChecker::check(&ast);

    // Resolver
    let symbol_table = build_symbol_table(&ast, is_std_lib);
    let ast = Resolver::resolve(ast, &symbol_table)?;
    
    // Type checker
    let ast = TypeChecker::check(ast, &symbol_table);

    // XXX: This is inefficient and is only done because the AST might have changed 
    // as we are doing type inferrence. We would be able to remove this if we updated
    // the symbol table on the fly too
    let symbol_table = build_symbol_table(&ast, is_std_lib);

    Ok((ast, symbol_table))
}

fn build_symbol_table(ast: &Program, is_std_lib : bool) -> SymbolTable {
    let mut root_table = SymbolTable::new();

    // Add the low level standard library symbol table
    // XXX: We will also compile and add the high level library here, but we do not have any high level std library constructs yet
    // Once modules are implemented, this(stdlib) will then move upto the module layer
    if !is_std_lib {
        load_low_level_libraries_into_symbol_table(&mut root_table);
    }

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

    for (module_key, module) in ast.modules.iter() {
        let mut module_symbol_table = SymbolTable::new();
        load_local_functions_into_symbol_table(module, &mut module_symbol_table);
        table.insert(module_key.clone().into(), SymbolInformation::Table(module_symbol_table))
    }
}

fn load_low_level_libraries_into_symbol_table(table: &mut SymbolTable) {

    // Import std here
    // XXX: Should be a better way to fetch the absolute path here. 
    // May have to wait until proper module dependency graph is added
    let std_lib = std::fs::read_to_string("../../../std/lib.noir").unwrap();

    // Parse and add low level functions into a symbol table
    // We could define the AST for this in the host language
    
    let mut parser = crate::Parser::with_input(&std_lib);
    let (program) = parser.parse_program().unwrap();
    let (checked_program, std_table) = check_program(program, true).unwrap();
    // We do nothing with the checked program for two reasons: Every module should have a copy of std_lib

    table.insert("std".to_string().into(), SymbolInformation::Table(std_table));
}