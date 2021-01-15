
mod errors;
use errors::AnalyserError;

pub mod scope;

mod attribute_check;
use attribute_check::AttributeChecker;

mod resolve;
use resolve::Resolver;


mod type_check;
use type_check::TypeChecker;

use crate::{Ident, NoirPath, Path, hir::{Context, basic_data::FunctionId, crate_def_map::{ModuleDefId, ModuleId}, resolution::import::{ImportDirective, resolve_imports}}};

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
pub fn check_crates(context : &Context) -> Result<(), Vec<AnalyserError>> {

    // Attribute checker
    if let Err(err) = AttributeChecker::check_crates(context) {    
        return Err(vec![err])
    }; 

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

// Resolve `foo::bar` in foo::bar::call() to the module with the function
pub fn resolve_call_path(context: &Context, module_id : ModuleId, path : &Path) -> Option<FunctionId> {
        // lets package up the path into an ImportDirective and resolve it using that
        let import = ImportDirective {
            module_id : module_id.local_id,
            path : path.clone(),
            alias : None,
        };
        let (unresolved, resolved) = resolve_imports(module_id.krate, vec![import], &context.def_maps);
        if !unresolved.is_empty() {
            return None
        }
        
        let import = match resolved.len() {
            0 => unreachable!("ice: unresolved and resolved are disjoint options. They cannot both be empty."),
            1 => {
                &resolved[0]
            },
            _ => unreachable!("ice: only one import directive was used as input.")
        };

        let mod_def_id = import.resolved_namespace.take_values()?;
        match mod_def_id {
            ModuleDefId::ModuleId(_) => None,
            ModuleDefId::FunctionId(func_id) => Some(func_id)
        }
}