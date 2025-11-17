use noirc_errors::Location;
use noirc_frontend::{
    ast::{Ident, ItemVisibility},
    graph::CrateGraph,
    hir::{
        def_map::{DefMaps, ModuleDefId, ModuleId},
        resolution::visibility::module_def_id_visibility,
    },
    node_interner::{DefinitionKind, FuncId, NodeInterner, TraitId, TypeId},
};
use regex::Regex;

use crate::{convert_primitive_type, items::PrimitiveTypeKind};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Link {
    pub target: LinkTarget,
    pub path: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LinkTarget {
    TopLevelItem(ModuleDefId),
    Method(ModuleDefId, FuncId),
    PrimitiveType(PrimitiveTypeKind),
    PrimitiveTypeFunction(PrimitiveTypeKind, FuncId),
}

#[derive(Clone, Copy)]
pub(crate) enum CurrentType {
    Type(TypeId),
    Trait(TraitId),
    PrimitiveType(PrimitiveTypeKind),
}

pub(crate) struct LinkFinder {
    /// A regex to match `[.+]` references.
    reference_regex: Regex,
}

impl LinkFinder {
    pub(crate) fn new() -> Self {
        let reference_regex = Regex::new(r"\[([^\[\]]+)\]").unwrap();
        Self { reference_regex }
    }

    pub(crate) fn find_links_in_markdown_line(
        &self,
        line: &str,
        current_module_id: ModuleId,
        current_type: Option<CurrentType>,
        interner: &NodeInterner,
        def_maps: &DefMaps,
        crate_graph: &CrateGraph,
    ) -> Vec<Link> {
        let captures = self
            .reference_regex
            .captures_iter(line)
            .map(|captures| {
                let start = captures.get(0).unwrap().start();
                let end = captures.get(0).unwrap().end();
                (captures[1].to_string(), start, end)
            })
            .collect::<Vec<_>>();

        let mut links = Vec::new();

        for (word, start, end) in captures {
            // Remove surrounding backticks if present.
            // The footnote will still mention the word with backticks.
            let path = &word;
            let path = path.strip_prefix('`').unwrap_or(path);
            let path = path.strip_suffix('`').unwrap_or(path);

            if let Some(target) = path_to_link_target(
                path,
                current_module_id,
                current_type,
                interner,
                def_maps,
                crate_graph,
            ) {
                links.push(Link { target, path: word.clone(), start, end });
            }
        }

        links
    }
}

/// Tries to convert a path into a link by resolving a path like `std::collections::Vec`.
/// This is similar to how name resolution works in the compiler, except that it's simpler
/// (no need to report errors), and references to type and trait functions are handled
/// a bit differently.
pub(crate) fn path_to_link_target(
    path: &str,
    current_module_id: ModuleId,
    current_type: Option<CurrentType>,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    crate_graph: &CrateGraph,
) -> Option<LinkTarget> {
    if path.is_empty() || path.contains(' ') {
        return None;
    }

    let segments: Vec<&str> = path.split("::").collect();

    if let Some(current_type) = current_type {
        if segments.len() <= 2 && segments[0] == "Self" {
            let method_name = segments.get(1).copied();
            match current_type {
                CurrentType::Type(type_id) => {
                    if let Some(method_name) = method_name {
                        return type_method_link_target(type_id, method_name, interner);
                    } else {
                        return Some(LinkTarget::TopLevelItem(ModuleDefId::TypeId(type_id)));
                    }
                }
                CurrentType::Trait(trait_id) => {
                    if let Some(method_name) = method_name {
                        return trait_method_link_target(trait_id, method_name, interner);
                    } else {
                        return Some(LinkTarget::TopLevelItem(ModuleDefId::TraitId(trait_id)));
                    }
                }
                CurrentType::PrimitiveType(primitive_type) => {
                    let Some(method_name) = method_name else {
                        return Some(LinkTarget::PrimitiveType(primitive_type));
                    };

                    // Array and Slice need special handling because they are composite types
                    // that aren't named like they are in the docs.
                    match primitive_type {
                        PrimitiveTypeKind::Array => {
                            let typ = noirc_frontend::Type::Array(
                                Box::new(noirc_frontend::Type::Error),
                                Box::new(noirc_frontend::Type::Error),
                            );
                            return primitive_type_method_link_target(
                                primitive_type,
                                &typ,
                                method_name,
                                interner,
                            );
                        }
                        PrimitiveTypeKind::Slice => {
                            let typ =
                                noirc_frontend::Type::Slice(Box::new(noirc_frontend::Type::Error));
                            return primitive_type_method_link_target(
                                primitive_type,
                                &typ,
                                method_name,
                                interner,
                            );
                        }
                        _ => {
                            let name = primitive_type.to_string();
                            return primitive_type_or_primitive_type_method_link_target(
                                &name,
                                Some(method_name),
                                interner,
                            );
                        }
                    }
                }
            }
        }
    }

    let check_dependencies = true;
    if let Some(link) = path_to_link_target_searching_modules(
        path,
        current_module_id,
        check_dependencies,
        interner,
        def_maps,
        crate_graph,
    ) {
        return Some(link);
    }

    // Search a primitive type or primitive type function
    if segments.len() > 2 {
        return None;
    }

    let name = segments[0];
    let method_name = segments.get(1).copied();
    primitive_type_or_primitive_type_method_link_target(name, method_name, interner)
}

fn path_to_link_target_searching_modules(
    path: &str,
    module_id: ModuleId,
    check_dependencies: bool,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    crate_graph: &CrateGraph,
) -> Option<LinkTarget> {
    // The path can be empty if a link is, for example, `[std]`.
    // In that case we'll recurse into this function with an empty path,
    // by searching starting from the `std` root module.
    if path.is_empty() {
        return Some(LinkTarget::TopLevelItem(ModuleDefId::ModuleId(module_id)));
    }

    let mut segments: Vec<&str> = path.split("::").collect();
    if let Some(first_segment) = segments.first() {
        if check_dependencies && *first_segment == "crate" {
            let crate_def_map = &def_maps[&module_id.krate];
            let root_local_module = crate_def_map.root();
            let root_module = ModuleId { krate: module_id.krate, local_id: root_local_module };
            segments.remove(0);
            let path = segments.join("::");
            return path_to_link_target_searching_modules(
                &path,
                root_module,
                false,
                interner,
                def_maps,
                crate_graph,
            );
        }
    }

    let crate_id = module_id.krate;
    let crate_def_map = &def_maps[&crate_id];
    let mut current_module = &crate_def_map[module_id.local_id];

    for (index, segment) in segments.iter().enumerate() {
        let name = Ident::new(segment.to_string(), Location::dummy());
        let per_ns = current_module.scope().find_name(&name);

        if per_ns.is_none() {
            // If we can't find the first segment we can try to search in dependencies
            if index == 0 && check_dependencies {
                let crate_data = &crate_graph[crate_id];
                let dependency_crate_id =
                    crate_data.dependencies.iter().find_map(|dependency| {
                        if &dependency.as_name() == segment {
                            Some(dependency.crate_id)
                        } else {
                            None
                        }
                    })?;
                let dependency_local_module_id = def_maps[&dependency_crate_id].root();
                let dependency_module_id =
                    ModuleId { krate: dependency_crate_id, local_id: dependency_local_module_id };
                segments.remove(0);
                let path = segments.join("::");
                return path_to_link_target_searching_modules(
                    &path,
                    dependency_module_id,
                    false,
                    interner,
                    def_maps,
                    crate_graph,
                );
            }

            return None;
        }

        // We are at the last segment so we can return the item if it's public
        if index == segments.len() - 1 {
            let (module_def_id, _, _) = per_ns.iter_items().next()?;
            let visibility = module_def_id_visibility(module_def_id, interner);
            if visibility != ItemVisibility::Public {
                return None;
            }
            return Some(LinkTarget::TopLevelItem(module_def_id));
        }

        // We are not at the last segment. Find a module, type or trait to continue.
        let (module_def_id, _, _) = per_ns.types?;
        match module_def_id {
            ModuleDefId::ModuleId(module_id) => {
                current_module = &crate_def_map[module_id.local_id];
            }
            ModuleDefId::TypeId(type_id) => {
                // This must refer to a type method, so only one segment should remain
                if index != segments.len() - 2 {
                    return None;
                }
                let method_name = segments.last().unwrap();
                return type_method_link_target(type_id, method_name, interner);
            }
            ModuleDefId::TraitId(trait_id) => {
                // This must refer to a trait method, so only one segment should remain
                if index != segments.len() - 2 {
                    return None;
                }
                let method_name = segments.last().unwrap();
                return trait_method_link_target(trait_id, method_name, interner);
            }
            ModuleDefId::TypeAliasId(_) => {
                // We could handle methods via type aliases, but for now we don't
                return None;
            }
            ModuleDefId::TraitAssociatedTypeId(..)
            | ModuleDefId::FunctionId(..)
            | ModuleDefId::GlobalId(..) => return None,
        }
    }
    None
}

fn type_method_link_target(
    type_id: TypeId,
    method_name: &str,
    interner: &NodeInterner,
) -> Option<LinkTarget> {
    let data_type = interner.get_type(type_id);
    if data_type.borrow().visibility != ItemVisibility::Public {
        return None;
    }
    let generic_types = data_type.borrow().generic_types();
    let typ = noirc_frontend::Type::DataType(data_type, generic_types);
    let methods = interner.get_type_methods(&typ)?;
    let method = methods.get(method_name)?;
    let method = method.direct.first()?;
    let method = method.method;
    Some(LinkTarget::Method(ModuleDefId::TypeId(type_id), method))
}

fn trait_method_link_target(
    trait_id: TraitId,
    method_name: &str,
    interner: &NodeInterner,
) -> Option<LinkTarget> {
    let trait_ = interner.get_trait(trait_id);
    if trait_.visibility != ItemVisibility::Public {
        return None;
    }
    let definition_id = trait_.find_method(method_name, interner)?;
    let definition = interner.definition(definition_id);
    if let DefinitionKind::Function(func_id) = definition.kind {
        Some(LinkTarget::Method(ModuleDefId::TraitId(trait_id), func_id))
    } else {
        None
    }
}

fn primitive_type_or_primitive_type_method_link_target(
    name: &str,
    method_name: Option<&str>,
    interner: &NodeInterner,
) -> Option<LinkTarget> {
    let primitive_type = noirc_frontend::elaborator::PrimitiveType::lookup_by_name(name)?;
    let doc_primitive_type = convert_primitive_type(primitive_type);
    let Some(method_name) = method_name else {
        return Some(LinkTarget::PrimitiveType(doc_primitive_type));
    };

    let typ = primitive_type.to_type();
    primitive_type_method_link_target(doc_primitive_type, &typ, method_name, interner)
}

fn primitive_type_method_link_target(
    primitive_type: PrimitiveTypeKind,
    typ: &noirc_frontend::Type,
    method_name: &str,
    interner: &NodeInterner,
) -> Option<LinkTarget> {
    let func_id = interner.lookup_direct_method(typ, method_name, false)?;
    Some(LinkTarget::PrimitiveTypeFunction(primitive_type, func_id))
}
