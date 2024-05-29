use noirc_errors::Spanned;

use crate::ast::ERROR_IDENT;
use crate::hir::def_map::{LocalModuleId, ModuleId};
use crate::hir::resolution::path_resolver::{PathResolver, StandardPathResolver};
use crate::hir::resolution::resolver::SELF_TYPE_NAME;
use crate::hir::scope::{Scope as GenericScope, ScopeTree as GenericScopeTree};
use crate::macros_api::Ident;
use crate::{
    hir::{
        def_map::{ModuleDefId, TryFromModuleDefId},
        resolution::errors::ResolverError,
    },
    hir_def::{
        expr::{HirCapturedVar, HirIdent},
        traits::Trait,
    },
    macros_api::{Path, StructId},
    node_interner::{DefinitionId, TraitId},
    Shared, StructType,
};
use crate::{Type, TypeAlias};

use super::{Elaborator, ResolverMeta};

type Scope = GenericScope<String, ResolverMeta>;
type ScopeTree = GenericScopeTree<String, ResolverMeta>;

impl<'context> Elaborator<'context> {
    pub(super) fn lookup<T: TryFromModuleDefId>(&mut self, path: Path) -> Result<T, ResolverError> {
        let span = path.span();
        let id = self.resolve_path(path)?;
        T::try_from(id).ok_or_else(|| ResolverError::Expected {
            expected: T::description(),
            got: id.as_str().to_owned(),
            span,
        })
    }

    pub(super) fn module_id(&self) -> ModuleId {
        assert_ne!(self.local_module, LocalModuleId::dummy_id(), "local_module is unset");
        ModuleId { krate: self.crate_id, local_id: self.local_module }
    }

    pub(super) fn resolve_path(&mut self, path: Path) -> Result<ModuleDefId, ResolverError> {
        let resolver = StandardPathResolver::new(self.module_id());
        let path_resolution = resolver.resolve(self.def_maps, path)?;

        if let Some(error) = path_resolution.error {
            self.push_err(error);
        }

        Ok(path_resolution.module_def_id)
    }

    pub(super) fn get_struct(&self, type_id: StructId) -> Shared<StructType> {
        self.interner.get_struct(type_id)
    }

    pub(super) fn get_trait_mut(&mut self, trait_id: TraitId) -> &mut Trait {
        self.interner.get_trait_mut(trait_id)
    }

    pub(super) fn resolve_local_variable(&mut self, hir_ident: HirIdent, var_scope_index: usize) {
        let mut transitive_capture_index: Option<usize> = None;

        for lambda_index in 0..self.lambda_stack.len() {
            if self.lambda_stack[lambda_index].scope_index > var_scope_index {
                // Beware: the same variable may be captured multiple times, so we check
                // for its presence before adding the capture below.
                let position = self.lambda_stack[lambda_index]
                    .captures
                    .iter()
                    .position(|capture| capture.ident.id == hir_ident.id);

                if position.is_none() {
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
                    transitive_capture_index = Some(position.unwrap_or(
                        // If this was a fresh capture, we added it to the end of
                        // the captures vector:
                        self.lambda_stack[lambda_index].captures.len() - 1,
                    ));
                }
            }
        }
    }

    pub(super) fn lookup_global(&mut self, path: Path) -> Result<DefinitionId, ResolverError> {
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

    pub fn push_scope(&mut self) {
        self.scopes.start_scope();
    }

    pub fn pop_scope(&mut self) {
        let scope = self.scopes.end_scope();
        self.check_for_unused_variables_in_scope_tree(scope.into());
    }

    pub fn check_for_unused_variables_in_scope_tree(&mut self, scope_decls: ScopeTree) {
        let mut unused_vars = Vec::new();
        for scope in scope_decls.0.into_iter() {
            Self::check_for_unused_variables_in_local_scope(scope, &mut unused_vars);
        }

        for unused_var in unused_vars.iter() {
            if let Some(definition_info) = self.interner.try_definition(unused_var.id) {
                let name = &definition_info.name;
                if name != ERROR_IDENT && !definition_info.is_global() {
                    let ident = Ident(Spanned::from(unused_var.location.span, name.to_owned()));
                    self.push_err(ResolverError::UnusedVariable { ident });
                }
            }
        }
    }

    fn check_for_unused_variables_in_local_scope(decl_map: Scope, unused_vars: &mut Vec<HirIdent>) {
        let unused_variables = decl_map.filter(|(variable_name, metadata)| {
            let has_underscore_prefix = variable_name.starts_with('_'); // XXX: This is used for development mode, and will be removed
            metadata.warn_if_unused && metadata.num_times_used == 0 && !has_underscore_prefix
        });
        unused_vars.extend(unused_variables.map(|(_, meta)| meta.ident.clone()));
    }

    /// Lookup a given trait by name/path.
    pub fn lookup_trait_or_error(&mut self, path: Path) -> Option<&mut Trait> {
        match self.lookup(path) {
            Ok(trait_id) => Some(self.get_trait_mut(trait_id)),
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Lookup a given struct type by name.
    pub fn lookup_struct_or_error(&mut self, path: Path) -> Option<Shared<StructType>> {
        match self.lookup(path) {
            Ok(struct_id) => Some(self.get_struct(struct_id)),
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Looks up a given type by name.
    /// This will also instantiate any struct types found.
    pub(super) fn lookup_type_or_error(&mut self, path: Path) -> Option<Type> {
        let ident = path.as_ident();
        if ident.map_or(false, |i| i == SELF_TYPE_NAME) {
            if let Some(typ) = &self.self_type {
                return Some(typ.clone());
            }
        }

        match self.lookup(path) {
            Ok(struct_id) => {
                let struct_type = self.get_struct(struct_id);
                let generics = struct_type.borrow().instantiate(self.interner);
                Some(Type::Struct(struct_type, generics))
            }
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    pub fn lookup_type_alias(&mut self, path: Path) -> Option<Shared<TypeAlias>> {
        self.lookup(path).ok().map(|id| self.interner.get_type_alias(id))
    }
}
