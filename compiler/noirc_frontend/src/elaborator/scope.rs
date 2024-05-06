use rustc_hash::FxHashMap as HashMap;

use crate::{macros_api::{StructId, Path}, node_interner::{TypeAliasId, DefinitionId, TraitId}, hir::{def_map::{TryFromModuleDefId, ModuleDefId}, resolution::errors::ResolverError}, Shared, StructType, hir_def::{traits::Trait, expr::{HirIdent, HirCapturedVar}}};
use crate::hir::comptime::Value;

use super::Elaborator;

#[derive(Default)]
pub(super) struct Scope {
    types: HashMap<String, TypeId>,
    values: HashMap<String, DefinitionId>,
    comptime_values: HashMap<String, Value>,
}

pub(super) enum TypeId {
    Struct(StructId),
    Alias(TypeAliasId),
}

impl Elaborator {
    pub fn lookup<T: TryFromModuleDefId>(&mut self, path: Path) -> Result<T, ResolverError> {
        let span = path.span();
        let id = self.resolve_path(path)?;
        T::try_from(id).ok_or_else(|| ResolverError::Expected {
            expected: T::description(),
            got: id.as_str().to_owned(),
            span,
        })
    }

    pub fn resolve_path(&mut self, path: Path) -> Result<ModuleDefId, ResolverError> {
        let path_resolution = self.path_resolver.resolve(&self.def_maps, path)?;

        if let Some(error) = path_resolution.error {
            self.push_err(error);
        }

        Ok(path_resolution.module_def_id)
    }

    pub fn get_struct(&self, type_id: StructId) -> Shared<StructType> {
        self.interner.get_struct(type_id)
    }

    pub fn get_trait_mut(&mut self, trait_id: TraitId) -> &mut Trait {
        self.interner.get_trait_mut(trait_id)
    }

    pub fn resolve_local_variable(&mut self, hir_ident: HirIdent, var_scope_index: usize) {
        let mut transitive_capture_index: Option<usize> = None;

        for lambda_index in 0..self.lambda_stack.len() {
            if self.lambda_stack[lambda_index].scope_index > var_scope_index {
                // Beware: the same variable may be captured multiple times, so we check
                // for its presence before adding the capture below.
                let pos = self.lambda_stack[lambda_index]
                    .captures
                    .iter()
                    .position(|capture| capture.ident.id == hir_ident.id);

                if pos.is_none() {
                    self.lambda_stack[lambda_index].captures.push(HirCapturedVar {
                        ident: hir_ident.clone(),
                        transitive_capture_index,
                    });
                }

                if lambda_index + 1 < self.lambda_stack.len() {
                    // There is more than one closure between the current scope and
                    // the scope of the variable, so this is a propagated capture.
                    // We need to track the transitive capture index as we go up in
                    // the closure stack.
                    transitive_capture_index = Some(pos.unwrap_or(
                        // If this was a fresh capture, we added it to the end of
                        // the captures vector:
                        self.lambda_stack[lambda_index].captures.len() - 1,
                    ));
                }
            }
        }
    }

    pub fn lookup_global(&mut self, path: Path) -> Result<DefinitionId, ResolverError> {
        let span = path.span();
        let id = self.resolve_path(path)?;

        if let Some(function) = TryFromModuleDefId::try_from(id) {
            return Ok(self.interner.function_definition_id(function));
        }

        if let Some(global) = TryFromModuleDefId::try_from(id) {
            let global = self.interner.get_global(global);
            return Ok(global.definition_id);
        }

        let expected = "global variable".into();
        let got = "local variable".into();
        Err(ResolverError::Expected { span, expected, got })
    }
}
