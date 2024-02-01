use std::collections::BTreeMap;

use fm::FileId;
use iter_extended::vecmap;

use crate::{
    graph::CrateId,
    hir::{
        def_collector::dc_crate::{CompilationError, UnresolvedStruct},
        def_map::ModuleId,
        Context,
    },
    node_interner::StructId,
    Generics, Ident, Type,
};

use super::{errors::ResolverError, path_resolver::StandardPathResolver, resolver::Resolver};

/// Create the mappings from TypeId -> StructType
/// so that expressions can access the fields of structs
pub(crate) fn resolve_structs(
    context: &mut Context,
    structs: BTreeMap<StructId, UnresolvedStruct>,
    crate_id: CrateId,
) -> Vec<(CompilationError, FileId)> {
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    // This is necessary to avoid cloning the entire struct map
    // when adding checks after each struct field is resolved.
    let struct_ids = structs.keys().copied().collect::<Vec<_>>();

    // Resolve each field in each struct.
    // Each struct should already be present in the NodeInterner after def collection.
    for (type_id, typ) in structs {
        let file_id = typ.file_id;
        let (generics, fields, resolver_errors) =
            resolve_struct_fields(context, crate_id, type_id, typ);
        errors.extend(vecmap(resolver_errors, |err| (err.into(), file_id)));
        context.def_interner.update_struct(type_id, |struct_def| {
            struct_def.set_fields(fields);
            struct_def.generics = generics;
        });
    }

    // Check whether the struct fields have nested slices
    // We need to check after all structs are resolved to
    // make sure every struct's fields is accurately set.
    for id in struct_ids {
        let struct_type = context.def_interner.get_struct(id);
        // Only handle structs without generics as any generics args will be checked
        // after monomorphization when performing SSA codegen
        if struct_type.borrow().generics.is_empty() {
            let fields = struct_type.borrow().get_fields(&[]);
            for field in fields.iter() {
                if field.1.is_nested_slice() {
                    errors.push((
                        ResolverError::NestedSlices { span: struct_type.borrow().location.span }
                            .into(),
                        struct_type.borrow().location.file,
                    ));
                }
            }
        }
    }

    errors
}

fn resolve_struct_fields(
    context: &mut Context,
    krate: CrateId,
    type_id: StructId,
    unresolved: UnresolvedStruct,
) -> (Generics, Vec<(Ident, Type)>, Vec<ResolverError>) {
    let path_resolver =
        StandardPathResolver::new(ModuleId { local_id: unresolved.module_id, krate });
    let file_id = unresolved.file_id;
    let (generics, fields, errors) =
        Resolver::new(&mut context.def_interner, &path_resolver, &context.def_maps, file_id)
            .resolve_struct_fields(unresolved.struct_def);

    // Register any other types used by this struct as a dependency in the dependency graph
    // TODO: Need to move this to while we're resolving an UnresolvedType, not after.
    // Otherwise we can't see global variable dependencies.
    for (_, field) in &fields {
        add_type_dependency(context, type_id, field);
    }

    (generics, fields, errors)
}

fn add_type_dependency(context: &mut Context, struct_id: StructId, typ: &Type) {
    match typ {
        Type::FieldElement
        | Type::Integer(_, _)
        | Type::Bool
        | Type::Unit
        | Type::NotConstant
        | Type::Constant(_)
        | Type::Error => (),

        // We don't count traits as dependencies since the type represented
        // by an impl trait is opaque and not needed until type checking.
        Type::TraitAsType(_, _, _) => (),

        Type::Array(length, elem) | Type::FmtString(length, elem) => {
            add_type_dependency(context, struct_id, length);
            add_type_dependency(context, struct_id, elem);
        }

        Type::String(length) => {
            add_type_dependency(context, struct_id, length);
        }
        Type::Struct(other_struct, _) => {
            let dependency_id = other_struct.borrow().id;
            context.def_interner.add_type_dependency(struct_id, dependency_id);
        }
        Type::Tuple(fields) => {
            for field in fields {
                add_type_dependency(context, struct_id, field);
            }
        }
        Type::TypeVariable(variable, _) | Type::NamedGeneric(variable, _) => {
            if let crate::TypeBinding::Bound(typ) = &*variable.borrow() {
                add_type_dependency(context, struct_id, typ);
            }
        }
        Type::Function(args, result, environment) => {
            for arg in args {
                add_type_dependency(context, struct_id, arg);
            }
            add_type_dependency(context, struct_id, result);
            add_type_dependency(context, struct_id, environment);
        }
        Type::MutableReference(element) => {
            add_type_dependency(context, struct_id, element);
        }
        Type::Forall(_, typ) => {
            add_type_dependency(context, struct_id, typ);
        }
    }
}
