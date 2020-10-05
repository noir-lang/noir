use super::macros::*;
use libnoirc_ast::{Ident, SymbolInformation,NoirFunction, SymbolTable, Type, NoirPath};


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
    pub fn return_type(&self) -> Type {
        match self {
            HashLibrary::SHA256 => Type::Array(2, Box::new(Type::Witness)),
            HashLibrary::AES => Type::Error, // Not implemented  yet
        }
    }

    // XXX: Until there is a proper module system, this is the workaround
    // Returns the path, the function name and the return type
    pub fn return_types() -> Vec<(NoirPath, Ident, Type)> {
        HashLibrary::iter().map(|variant| {
            (NoirPath::External(vec![Ident("std".into()), Ident("hash".into())]), variant.to_string(), variant.return_type())
        }).collect()
    }
    
    pub fn symbol_table() -> SymbolTable {
        let mut hash_symbol_table = SymbolTable::new();
        for variant in HashLibrary::iter() {
            hash_symbol_table.insert_foreign_func(variant.to_string());
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