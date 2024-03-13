use iter_extended::vecmap;
use noirc_frontend::{
    graph::CrateId,
    hir::def_collector::dc_crate::UnresolvedTraitImpl,
    macros_api::{HirContext, ModuleDefId, StructId},
    node_interner::{TraitId, TraitImplId},
    Signedness, Type, UnresolvedTypeData,
};

pub fn collect_crate_structs(crate_id: &CrateId, context: &HirContext) -> Vec<StructId> {
    context
        .def_map(crate_id)
        .expect("ICE: Missing crate in def_map")
        .modules()
        .iter()
        .flat_map(|(_, module)| {
            module.type_definitions().filter_map(|typ| {
                if let ModuleDefId::TypeId(struct_id) = typ {
                    Some(struct_id)
                } else {
                    None
                }
            })
        })
        .collect()
}

pub fn collect_traits(context: &HirContext) -> Vec<TraitId> {
    let crates = context.crates();
    crates
        .flat_map(|crate_id| context.def_map(&crate_id).map(|def_map| def_map.modules()))
        .flatten()
        .flat_map(|module| {
            module.type_definitions().filter_map(|typ| {
                if let ModuleDefId::TraitId(struct_id) = typ {
                    Some(struct_id)
                } else {
                    None
                }
            })
        })
        .collect()
}

/// Computes the aztec signature for a resolved type.
pub fn signature_of_type(typ: &Type) -> String {
    match typ {
        Type::Integer(Signedness::Signed, bit_size) => format!("i{}", bit_size),
        Type::Integer(Signedness::Unsigned, bit_size) => format!("u{}", bit_size),
        Type::FieldElement => "Field".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::Array(len, typ) => {
            if let Type::Constant(len) = **len {
                format!("[{};{len}]", signature_of_type(typ))
            } else {
                unimplemented!("Cannot generate signature for array with length type {:?}", typ)
            }
        }
        Type::Struct(def, args) => {
            let fields = def.borrow().get_fields(args);
            let fields = vecmap(fields, |(_, typ)| signature_of_type(&typ));
            format!("({})", fields.join(","))
        }
        Type::Tuple(types) => {
            let fields = vecmap(types, signature_of_type);
            format!("({})", fields.join(","))
        }
        _ => unimplemented!("Cannot generate signature for type {:?}", typ),
    }
}

// Fetches the name of all structs that implement trait_name, both in the current crate and all of its dependencies.
pub fn fetch_struct_trait_impls(
    context: &mut HirContext,
    unresolved_traits_impls: &[UnresolvedTraitImpl],
    trait_name: &str,
) -> Vec<String> {
    let mut struct_typenames: Vec<String> = Vec::new();

    // These structs can be declared in either external crates or the current one. External crates that contain
    // dependencies have already been processed and resolved, but are available here via the NodeInterner. Note that
    // crates on which the current crate does not depend on may not have been processed, and will be ignored.
    for trait_impl_id in 0..context.def_interner.next_trait_impl_id().0 {
        let trait_impl = &context.def_interner.get_trait_implementation(TraitImplId(trait_impl_id));

        if trait_impl.borrow().ident.0.contents == *trait_name {
            if let Type::Struct(s, _) = &trait_impl.borrow().typ {
                struct_typenames.push(s.borrow().name.0.contents.clone());
            } else {
                panic!("Found impl for {} on non-Struct", trait_name);
            }
        }
    }

    // This crate's traits and impls have not yet been resolved, so we look for impls in unresolved_trait_impls.
    struct_typenames.extend(
        unresolved_traits_impls
            .iter()
            .filter(|trait_impl| {
                trait_impl
                    .trait_path
                    .segments
                    .last()
                    .expect("ICE: empty trait_impl path")
                    .0
                    .contents
                    == *trait_name
            })
            .filter_map(|trait_impl| match &trait_impl.object_type.typ {
                UnresolvedTypeData::Named(path, _, _) => {
                    Some(path.segments.last().unwrap().0.contents.clone())
                }
                _ => None,
            }),
    );

    struct_typenames
}
