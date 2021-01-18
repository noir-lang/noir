use arena::Index;

use crate::{BlockStatement, FunctionKind, Ident, NoirFunction, Type, token::Attribute};

use super::lower::def_interner::FuncId;

pub type FunctionId = FuncId;

/// Stores information about the functions basic data
#[derive(Debug)]
pub struct FunctionArena {
    arena_basic_data : arena::Arena<FunctionBasicData>,
    arena_data : arena::Arena<BlockStatement>, // XXX: We only need to store the functionBody Here as everything else is in the BasicData. We can generalise and make it DefWithBody
    defs : Vec<FunctionId>,
}

impl Default for FunctionArena {
    fn default() -> Self {
        FunctionArena {
            arena_basic_data : arena::Arena::default(),
            arena_data : arena::Arena::default(),
            defs : Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionBasicData {
    pub name : String,

    pub kind : FunctionKind,

    pub attributes : Option<Attribute>,
    pub parameters : Vec<(Ident, Type)>, // XXX: We don't need the Ident here as we are not checking the Body. But we will need it later on unless we use De Bruijn indices
    pub return_type : Type,

    pub body_id : Option<arena::Index>,
}

impl FunctionArena {

    pub fn ast_func_to_func_data(&mut self, func : NoirFunction) -> FunctionId {

        let name = func.name().to_owned();
        let attributes = func.attribute().cloned();
        let parameters = func.parameters().to_owned();
        let return_type = func.return_type();

        let mut func_idx = None;
        if func.def.body.len() > 0 {
            let index = self.arena_data.insert(func.def.body);
            self.defs.push(FuncId(index));

            func_idx = Some(index);
        }

        let basic_data = FunctionBasicData {
            name,
            kind : func.kind, 
            attributes,
            parameters,
            return_type,
            body_id : func_idx,
        };
        
        FuncId(self.arena_basic_data.insert(basic_data))

    }
    pub fn basic_data(&self, id : FunctionId) -> &FunctionBasicData {
        &self.arena_basic_data[id.0]
    }
    pub fn data(&self, id : FunctionId) -> Option<&BlockStatement> {
        let basic_data = &self.arena_basic_data[id.0];
        let func_body_idx = basic_data.body_id?;
        Some(&self.arena_data[func_body_idx])
    }
    pub fn iter_keys(&self) -> Vec<FunctionId> {
        self.defs.clone()
    }
}

