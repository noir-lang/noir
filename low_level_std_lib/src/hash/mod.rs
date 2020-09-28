use super::macros::*;
use libnoirc_ast::{Ident, SymbolInformation, SymbolTable};


iterable_enum! {
    HashLibrary {SHA256,AES}
}

impl HashLibrary {
    pub fn to_string(&self) -> Ident {
        match self {
            HashLibrary::SHA256 => Ident("sha256".into()),
            HashLibrary::AES => Ident("aes".into())
        }
    }
    
    pub fn symbol_table() -> SymbolTable {
        let mut hash_symbol_table = SymbolTable::new();
        for variant in HashLibrary::iter() {
            hash_symbol_table.insert(variant.to_string(), SymbolInformation::LowLevelFunction);
        }
        hash_symbol_table
    }
    pub fn find(name : &Ident) -> Option<HashLibrary> {
        for variant in HashLibrary::iter() {
            if &variant.to_string() == name {
                return Some(variant)
            }
        }
        return None
    }
}