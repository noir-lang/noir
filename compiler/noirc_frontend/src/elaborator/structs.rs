//! Struct definition collection and field resolution.

use std::collections::BTreeMap;

use iter_extended::vecmap;

use crate::{
    StructField,
    ast::NoirStruct,
    hir::{def_collector::dc_crate::UnresolvedStruct, resolution::errors::ResolverError},
    node_interner::{DependencyId, ReferenceId, TypeId},
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Collects and resolves all struct definitions.
    ///
    /// This method performs several tasks:
    /// 1. Resolves the types of all struct fields
    /// 2. Validates visibility constraints (public structs cannot expose private types)
    /// 3. Registers LSP definition locations for IDE support
    /// 4. Checks for disallowed nested slice types
    ///
    /// Structs must already be interned from the earlier definition collection phase.
    /// This method fills in the field information for each struct.
    pub(super) fn collect_struct_definitions(
        &mut self,
        structs: &BTreeMap<TypeId, UnresolvedStruct>,
    ) {
        // This is necessary to avoid cloning the entire struct map
        // when adding checks after each struct field is resolved.
        let struct_ids = structs.keys().copied().collect::<Vec<_>>();

        // Resolve each field in each struct.
        // Each struct should already be present in the NodeInterner after def collection.
        for (type_id, typ) in structs {
            self.local_module = typ.module_id;

            let fields = self.resolve_struct_fields(&typ.struct_def, *type_id);

            if typ.struct_def.is_abi() {
                for field in &fields {
                    self.mark_type_as_used(&field.typ);
                }
            }

            self.check_struct_field_type_visibility(&typ.struct_def, &fields);

            if self.interner.is_in_lsp_mode() {
                for (field_index, field) in fields.iter().enumerate() {
                    let location = field.name.location();
                    let reference_id = ReferenceId::StructMember(*type_id, field_index);
                    self.interner.add_definition_location(reference_id, location);
                }
            }

            self.interner.update_type(*type_id, |struct_def| {
                struct_def.set_fields(fields);
            });
        }

        self.check_for_nested_slices(&struct_ids);
    }

    /// Resolves the field types for a single struct definition.
    ///
    /// This method:
    /// - Sets up the generic context from the struct's generic parameters
    /// - Resolves each field's type in the context of the struct's generics
    /// - Tracks the struct id to detect circular dependencies
    ///
    /// The generic scope is automatically recovered after resolution completes.
    fn resolve_struct_fields(
        &mut self,
        unresolved: &NoirStruct,
        struct_id: TypeId,
    ) -> Vec<StructField> {
        self.recover_generics(|this| {
            this.current_item = Some(DependencyId::Struct(struct_id));

            this.resolving_ids.insert(struct_id);

            let struct_def = this.interner.get_type(struct_id);
            this.add_existing_generics(&unresolved.generics, &struct_def.borrow().generics);

            let wildcard_allowed = false;
            let fields = vecmap(&unresolved.fields, |field| {
                let ident = &field.item.name;
                let typ = &field.item.typ;
                let visibility = field.item.visibility;
                StructField {
                    visibility,
                    name: ident.clone(),
                    typ: this.resolve_type(typ.clone(), wildcard_allowed),
                }
            });

            this.resolving_ids.remove(&struct_id);

            fields
        })
    }

    /// Checks all resolved structs for nested slice types, which are not allowed.
    ///
    /// This check must happen after all struct fields are resolved to ensure we have
    /// complete type information. We only check structs without generics here, as
    /// generic structs are validated after monomorphization during SSA codegen.
    fn check_for_nested_slices(&mut self, struct_ids: &[TypeId]) {
        for id in struct_ids {
            let struct_type = self.interner.get_type(*id);

            // Only handle structs without generics as any generics args will be checked
            // after monomorphization when performing SSA codegen
            if struct_type.borrow().generics.is_empty() {
                let fields = struct_type.borrow().get_fields(&[]).unwrap();
                for (_, field_type, _) in fields.iter() {
                    if field_type.is_nested_slice() {
                        let location = struct_type.borrow().location;
                        self.push_err(ResolverError::NestedSlices { location });
                    }
                }
            }
        }
    }
}
