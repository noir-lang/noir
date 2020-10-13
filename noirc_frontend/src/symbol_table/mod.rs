
use crate::{FunctionLiteral as Function, FunctionDefinition, Ident, Type, NoirPath, ArraySize};
use crate::lexer::token::Attribute;
use std::collections::HashMap;

// A NoirFunction can be either a function literal (closure), a foreign low level function or a function definition 
// A closure / function definition will be stored under a name, so we do not differentiate between their variants
// The name for function literal will be the variable it is binded to, and the name for a function definition will 
// be the function name itself.
#[derive(Clone, Debug)]
pub enum NoirFunction {
    LowLevelFunction(Function),
    Function(Function)
}

impl NoirFunction {
    pub fn return_type(&self) -> Type {
        match self {
            NoirFunction::Function(func_def) => func_def.return_type.clone(),
            NoirFunction::LowLevelFunction(func_def) => func_def.return_type.clone(),
        }
    }
}

impl Into<NoirFunction> for Function {
    fn into(self) -> NoirFunction {
        NoirFunction::Function(self)
    }
}

#[derive(Clone, Debug)]
pub enum SymbolInformation {
    Variable(Type), // Depth of symbol table, represents it's scope 
    Function(NoirFunction),
    Alias(String, Vec<String>), // Example: use std::hash::sha256 as hash_function => Alias("hash_func", ["std", "hash", "sha256"])
    Table(SymbolTable),
}

/// Stores all information about all of this modules symbols
#[derive(Clone, Debug)]
pub struct SymbolTable(HashMap<Ident, SymbolInformation>);

impl SymbolTable {
    pub fn new() -> SymbolTable {

        // XXX: Load low level std lib on every symbol table, once this is no longer in ast module

        SymbolTable(HashMap::new())
    }
    pub fn insert(&mut self, symbol: Ident, si: SymbolInformation) {
        self.0.insert(symbol, si);
    }
    pub fn insert_func_def(&mut self, func_def: &FunctionDefinition) {
        let (func_name, func) = parse_function_declaration(func_def);
        let function_already_declared =  self.look_up(&func_name).is_some(); 
        if function_already_declared {
            panic!("Another symbol has been defined with the name {}", &func_name.0)
        }
        let attribute = match &func_def.attribute{
            Some(attr) => attr,
            None => {
                self.insert(func_name.clone(), SymbolInformation::Function(func.clone().into()));
                return
            }
        };

        match attribute{
            Attribute::Foreign(_) => self.insert(func_name.clone(), SymbolInformation::Function(NoirFunction::LowLevelFunction(func.clone().into()))),
        };
    }
    pub fn update_func_def(&mut self, func_def: &FunctionDefinition) {
        let (func_name, func) = parse_function_declaration(func_def);
        let function_already_declared =  self.look_up(&func_name).is_some(); 
        if !function_already_declared {
            panic!("Cannot update func with name {} because it does not exist", &func_name.0)
        }
        self.insert(func_name.clone(), SymbolInformation::Function(func.clone().into()));
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
    // XXX: Get the function name from a constant and do not pass it in
    pub fn look_up_main_func(&self, func_name: &Ident) -> Option<Function> {

        let symbol = self.look_up(func_name);
        let symbol = match symbol {
            Some(symbol) => symbol,
            None => return None
        };

        let noir_func : &NoirFunction = match symbol {
            SymbolInformation::Function(noir_func) => noir_func,
            _=> return None,
        };

        match noir_func {
            NoirFunction::Function(main) => Some(main.clone()),
            _ => return None
        }

    }
    // XXX: Unfortunately, this is hiding the fact that the RHS which is a symbol information, can only be a LowLevelFunction
    // XXX: Possible create a enum for function, so that function can either be LowLevel or Compiled 
    pub fn look_up_func(&self, noir_path : NoirPath, func_name: &Ident) -> Option<NoirFunction> {

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

        let symbol_found = tbl.look_up(func_name);

        let valid_symbol = match symbol_found {
            Some(valid_symbol) => valid_symbol,
            None => panic!("Cannot find an object called {} under the path {}", &func_name.0, noir_path_string)
        };

        match valid_symbol {
            SymbolInformation::Function(noir_func) => Some(noir_func.clone()),
            _=> panic!("Cannot find a function called {} under the path {}", &func_name.0, noir_path_string)
        }
     
    }

    // XXX: Here we could possibly make this cheaper, by storing a function symbol in the Variable enum
    // It is a symbol for functions, so we can quickly check for their types and if they are valid
    // In variable append "fn_" + func_name -> Key = fn_{func_name} Value = Variable(ReturnType)
    pub fn valid_func(&self, noir_path : NoirPath, func_name: &Ident) -> bool {
        self.look_up_func(noir_path, func_name).is_some()
    }
}

/// Convert a function declarations into a function object
fn parse_function_declaration(func_dec: &FunctionDefinition) -> (Ident, Function) {
    let func_name = func_dec.name.clone();
    let body = func_dec.literal.body.clone();
    let return_type = func_dec.literal.return_type.clone();
    let parameters = func_dec.literal.parameters.clone();

    (func_name, Function { body, parameters, return_type })
}

// Note: We are implementing the symbol table so that later on, we can do imports properly
// We should get the standard library from the compiler and iteratively add them into the program here
// Instead of SymbolInformation::LowLevelLibrary use evaluator::LowLevelStdLibrary
// Also remember to fix up the useParser and test it works
#[test]
fn test_k() {
    let mut std_hash_st = SymbolTable::new();
    std_hash_st.insert(Ident("sha256".into()), SymbolInformation::Variable(Type::Bool));

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
}
