
mod errors;
use errors::AnalyserError;

pub mod scope;

mod attribute_check;
use attribute_check::AttributeChecker;

mod resolve;
use resolve::Resolver;


mod type_check;
use type_check::TypeChecker;

use crate::{Ident, NoirPath};

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
use crate::ast::ImportStatement;
use crate::parser::Program;
use crate::krate::crate_manager::{CrateID,CrateManager};
use crate::krate::crate_unit::{ModID, VirtualPath};

use std::path::PathBuf;

/// We assume that the standard library has already been loaded
pub fn check_crates(crate_manager: &mut CrateManager<Program>) -> Result<(), Vec<AnalyserError>> {

    // Attribute checker
    if let Err(err) = AttributeChecker::check_crates(&crate_manager) {    
        return Err(vec![err])
    }; 

    // Resolver
    // Lets do the resolver!
    // XXX: We are doing this because currently CrateIDs and ModIDs are not linked up
    let mut modules_to_update = Vec::new();
    for crate_id in crate_manager.crate_ids() {
        let krate = crate_manager.get_crate_with_id(crate_id).unwrap();
        for mod_id in krate.module_ids() {
            let mut module = krate.get_module(mod_id).unwrap().clone();
            
            Resolver::resolve(&mut module, mod_id, crate_id, crate_manager)?;
            TypeChecker::check(&mut module, mod_id, crate_id, crate_manager)?;

            modules_to_update.push((mod_id, crate_id, module));
        }
    }

    // Update the modules in the crate manager
    // This will be removed, once type checker does not work over the AST
    for module in modules_to_update.into_iter() {
        let mod_id = module.0;
        let crate_id = module.1;
        let module = module.2;
        
        let krate = crate_manager.get_mut_crate_with_id(crate_id).unwrap();
        let fetched_module = krate.get_mut_module(mod_id).unwrap();
        *fetched_module = module;
    }
    Ok(())
}

pub fn noir_path_to_virtual_path(noir_path : NoirPath) -> VirtualPath {
    match noir_path {
        NoirPath::Current => panic!("We might remove this as we do not know what current is, can just use 'crate/super' "),
        NoirPath::External(pth) => {
            
            let segments : String = pth.into_iter().map(|ident| {
                let mut segment = ident.0.contents.clone();
                segment.push_str("/");
                segment
            }).collect();
            
            VirtualPath::from_noir_path(PathBuf::from(segments))
        }
    }
}

// Resolve use `foo::bar as hello` to a (Key, ModId, CrateId)
// In this example, it would be the ModID and CrateID for the `bar` module and the key would be hello
fn resolve_import(import : &ImportStatement, crate_manager : &CrateManager<Program>) -> (Ident, ModID, CrateID) {
   todo!()
    // let vp = noir_path_to_virtual_path(import.path.clone().into());
    // crate_manager.debug_virtual_paths();
    // let (mod_id, crate_id, _) = crate_manager.find_module(vp).unwrap();
    // let key = match &import.alias {
    // Some(alias) => alias.to_owned(),
    //     None => import.path.last().unwrap().to_owned()
    // };
    // (key, mod_id, crate_id)
}