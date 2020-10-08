#[macro_use]
pub(crate) mod macros;
mod hash;

use noirc_frontend::ast::{Ident, NoirPath, Type};
use noirc_frontend::symbol_table::{SymbolInformation, SymbolTable};

pub use hash::HashLibrary;

/// These are the gadgets which are implemented directly in the underlying proof system
#[derive(Clone, Debug)]
pub enum LowLevelStandardLibrary {
    Hash(HashLibrary),
}

impl LowLevelStandardLibrary {
    pub fn to_string(&self) -> Ident {
        match self {
            LowLevelStandardLibrary::Hash(hl) => hl.to_string()
        }
    }
    pub fn find(name : &Ident) -> Option<LowLevelStandardLibrary> {

        // First search in HashLibrary
        match HashLibrary::find(name) {
            None => {},
            Some(hl) => return Some(LowLevelStandardLibrary::Hash(hl))
        };

        return None
    }

    pub fn return_types() -> Vec<(NoirPath, Ident, Type)> {
        HashLibrary::return_types()
    }
}

/// Populates symbol table with all of the low level standard library functions
/// XXX: Why do we have this? Because functions that are directly callable from the underlying proof system cannot be represented in the AST AFAICT
/// We don't want the parser to have logic to deal with this, so if there is a clean solution, it would involve the lexer 
/// One thing is for the analyser to check if the function being called is in the std_lib and then somehow statically ask std_lib if the function exists, then modify the AST 
/// so that the compiler can knows to call std_lib for that function 
pub fn populate_symbol_table(table : &mut SymbolTable) {
    // Add the std library stuff
    // However, I think we need a way for the analyser to parse a function as std library . We cannot make them functions because FuncLiterals need a BlockStatement as their body
    // We'd need to somehow use the path and the symbol information
    // This function will add all of the gadget definitions to the Circuit

    // Add low level hashes to the standard library symbol table symbol table
    let std_hash_st = HashLibrary::symbol_table();

    let mut std_st = SymbolTable::new();
    std_st.insert(Ident("hash".into()), SymbolInformation::Table(std_hash_st));

    // Add standard library symbol table to the table we wish to populate
    table.insert(Ident("std".into()), SymbolInformation::Table(std_st));
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}