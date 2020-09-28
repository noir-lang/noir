// XXX: Lets reformat our modules, so that we have a front-end workspace and the symbol table will simply be in a  module and npot under ast workspace
/*

front-end - workspace
 - lexer (mod)
 - parser (mod)
 - ast (mod)
 - symbol_table (mod)
 - analyser (mod)
*/
use crate::{FunctionLiteral as Function, Ident, Type, NoirPath};
use std::collections::HashMap;
// XXX: This will eventually replace the env.get() procedures in the evaluator
#[derive(Copy, Clone, Debug)]
pub struct Scope(usize);

impl Scope {
    pub fn global() -> Scope {
        Scope(0)
    }
    pub fn function() -> Scope {
        Scope(1)
    }
}

#[derive(Clone, Debug)]
pub enum SymbolInformation {
    Variable(Type, Scope),
    Function(Function),
    LowLevelFunction,
    Alias(String, Vec<String>), // Example: use std::hash::sha256 as hash_function => Alias("hash_func", ["std", "hash", "sha256"])
    Table(SymbolTable),
}

/// Stores all information about all of this modules symbols
#[derive(Clone, Debug)]
pub struct SymbolTable(HashMap<Ident, SymbolInformation>);

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable(HashMap::new())
    }
    pub fn insert(&mut self, symbol: Ident, si: SymbolInformation) {
        self.0.insert(symbol, si);
    }
    pub fn look_up(&self, symbol: &Ident) -> Option<&SymbolInformation> {
        self.0.get(symbol)
    }
    // XXX: Find a better way to do this path decent
    pub fn look_up_path(&self, mut path: NoirPath) -> Option<SymbolInformation> {
        // Traverse the path to find the symbol that we need

        let current_table = self.clone();

        if path == NoirPath::Current {
            return Some(SymbolInformation::Table(current_table));
        }

        let mut found_symbol: SymbolInformation = SymbolInformation::Table(current_table);
        while path.len() > 0 {
            let (first, rest) = path.split_first().unwrap();
            let new_symbol = match found_symbol.clone() {
                SymbolInformation::Table(tbl) => tbl.clone().look_up(first).cloned(),
                _ => panic!("unexpected item, expected a lookup table"),
            };

            match new_symbol {
                None => return None,
                Some(found) => found_symbol = found,
            };

            path = rest
        }

        Some(found_symbol)
    }

    // This assumes the main function is in the currrent path 
    pub fn look_up_main_func(&self, func_name: &Ident) -> Option<Function> {

        let symbol = self.look_up(func_name);
        let symbol = match symbol {
            Some(symbol) => symbol,
            None => return None
        };

        match symbol {
            SymbolInformation::Function(func) => Some(func.clone()),
            _=> None,
        }
    }
    // XXX: Unfortunately, this is hiding the fact that the RHS which is a symbol information, can only be a LowLevelFunction
    // XXX: Possible create a enum for function, so that function can either be LowLevel or Compiled 
    pub fn look_up_func(&self, noir_path : NoirPath, func_name: &Ident) -> (Option<Function>, Option<SymbolInformation>) {

        let noir_path_string = noir_path.to_string();

        let symbol_info = self.look_up_path(noir_path);

        let valid_symbol = match symbol_info {
            Some(valid_symbol) => valid_symbol,
            None => panic!("Cannot find a symbol under the path {}", noir_path_string)
        };

        let tbl = match valid_symbol {
            SymbolInformation::Table(tbl) => tbl,
            _=> panic!("expected a symbol table for the specified path")
        };

        let func_to_call = tbl.look_up(func_name);

        let valid_func = match func_to_call {
            Some(valid_func) => valid_func.clone(),
            None => panic!("Cannot find a function called {} under the path {}", &func_name.0, noir_path_string)
        };

        match valid_func {
            SymbolInformation::Function(func) => (Some(func), None),
            SymbolInformation::LowLevelFunction => (None, Some(SymbolInformation::LowLevelFunction)),
            _=> (None,None)
        }
    }

    pub fn valid_func(&self, noir_path : NoirPath, func_name: &Ident) -> bool {
        let (func_literal, low_level_func)  = self.look_up_func(noir_path, func_name);

        if func_literal.is_some() || low_level_func.is_some() {
            return true
        }

        return false

    }
}

// Note: We are implementing the symbol table so that later on, we can do imports properly
// We should get the standard library from the compiler and iteratively add them into the program here
// Instead of SymbolInformation::LowLevelLibrary use evaluator::LowLevelStdLibrary
// Also remember to fix up the useParser and test it works
#[test]
fn test_k() {
    let mut std_hash_st = SymbolTable::new();
    std_hash_st.insert(Ident("sha256".into()), SymbolInformation::LowLevelFunction);

    let mut std_st = SymbolTable::new();
    std_st.insert(Ident("hash".into()), SymbolInformation::Table(std_hash_st));

    let mut root_table = SymbolTable::new();
    root_table.insert(Ident("std".into()), SymbolInformation::Table(std_st));

    let path = vec![
        Ident("std".into()),
        Ident("hash".into()),
        Ident("sha256".into()),
    ];

    let result = root_table.look_up_path(path.into());
    dbg!(result);
}
