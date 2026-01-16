use noirc_driver::CrateId;
use noirc_errors::Location;
use noirc_frontend::{
    ast::Ident,
    graph::CrateGraph,
    hir::def_map::{DefMaps, ModuleDefId, ModuleId},
    modules::get_parent_module,
    node_interner::{DefinitionKind, FuncId, NodeInterner, TraitId, TypeId},
};
use regex::Regex;

use crate::{convert_primitive_type, items::PrimitiveTypeKind};

/// A resolved markdown link found in a line of markdown, in the form:
/// - `[name]` (`path` will be the same as `name`)
/// - `[name][path]`
/// - `[name](path)`
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Link {
    /// The link target. If None it means this is a broken link.
    pub target: Option<LinkTarget>,
    pub name: String,
    pub path: String,
    /// The line number in the comments where this link occurs (0-based).
    pub line: usize,
    /// The start byte of the link in the line.
    pub start: usize,
    /// The end byte of the link in the line.
    pub end: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum LinkTarget {
    TopLevelItem(ModuleDefId),
    Method(ModuleDefId, FuncId),
    StructMember(TypeId, usize),
    PrimitiveType(PrimitiveTypeKind),
    PrimitiveTypeFunction(PrimitiveTypeKind, FuncId),
}

#[derive(Clone, Copy)]
pub enum CurrentType {
    Type(TypeId),
    Trait(TraitId),
    PrimitiveType(PrimitiveTypeKind),
}

/// Finds links in markdown comments.
///
/// It skips links that are inside code blocks (as they are not links, really),
/// and for that it keeps track of whether we are inside a code block or not.
///
/// Use [`Self::reset`] to reset the state when starting to process a new group of doc comments.
pub struct LinkFinder {
    /// A regex to match `[.+]` references.
    reference_regex: Regex,
    in_code_block: bool,
}

impl Default for LinkFinder {
    fn default() -> Self {
        Self { reference_regex: reference_regex(), in_code_block: false }
    }
}

impl LinkFinder {
    pub fn find_links(
        &mut self,
        comments: &str,
        current_module_id: ModuleId,
        current_type: Option<CurrentType>,
        interner: &NodeInterner,
        def_maps: &DefMaps,
        crate_graph: &CrateGraph,
    ) -> Vec<Link> {
        let mut links = Vec::new();
        for (line_number, line) in comments.lines().enumerate() {
            let trimmed_line = line.trim_start();

            // Track block codes. We check "```", "* ```" and "/// ```" because the comments
            // given here might include the syntax for comments.
            if trimmed_line.starts_with("```")
                || trimmed_line.strip_prefix('*').is_some_and(|line| line.trim().starts_with("```"))
                || trimmed_line
                    .strip_prefix("///")
                    .is_some_and(|line| line.trim().starts_with("```"))
            {
                self.in_code_block = !self.in_code_block;
                continue;
            }
            if self.in_code_block {
                continue;
            }

            let line_links = self.find_links_in_line(
                line,
                line_number,
                current_module_id,
                current_type,
                interner,
                def_maps,
                crate_graph,
            );
            links.extend(line_links);
        }
        links
    }

    #[allow(clippy::too_many_arguments)]
    fn find_links_in_line(
        &mut self,
        line: &str,
        line_number: usize,
        current_module_id: ModuleId,
        current_type: Option<CurrentType>,
        interner: &NodeInterner,
        def_maps: &DefMaps,
        crate_graph: &CrateGraph,
    ) -> impl Iterator<Item = Link> {
        find_links_in_markdown_line(line, &self.reference_regex).map(move |link| {
            let path = &link.link;
            let target = path_to_link_target(
                path,
                current_module_id,
                current_type,
                interner,
                def_maps,
                crate_graph,
            );
            Link {
                target,
                line: line_number,
                name: link.name,
                path: link.link,
                start: link.start,
                end: link.end,
            }
        })
    }

    /// Resets the state, meaning that it won't consider itself to be inside a code block.
    pub fn reset(&mut self) {
        self.in_code_block = false;
    }
}

/// An unresolved markdown link found in a line of markdown, in the form:
/// - `[name]` (`link` will be the same as `name`)
/// - `[name][link]`
/// - `[name](link)`
///
/// It's unresolved because the link might not resolve to an actual item.
#[derive(Debug)]
struct PlainLink {
    pub name: String,
    pub link: String,
    /// The start byte of the link in the line.
    pub start: usize,
    /// The end byte of the link in the line.
    pub end: usize,
}

/// Finds links in a markdown line. Only links that look like Noir paths are returned.
/// For example, `[1 + 2]` will not be returned as a link, while `[foo::Bar]` will.
fn find_links_in_markdown_line(line: &str, regex: &Regex) -> impl Iterator<Item = PlainLink> {
    regex.captures_iter(line).filter_map(|captures| {
        let first_capture = captures.get(0).unwrap();
        let start = first_capture.start();
        let end = first_capture.end();
        let word = captures.get(1)?.as_str().to_string();
        let link = captures
            .get(2)
            .or(captures.get(3))
            .map(|capture| capture.as_str().to_string())
            .unwrap_or_else(|| word.clone());

        // If the left bracket it escaped (`\[`) then it's not a link.
        // There's no need to check the right bracket as `\` is not a valid path character.
        if start > 0 && line.chars().nth(start - 1).is_some_and(|char| char == '\\') {
            return None;
        }

        // Remove surrounding backticks if present.
        // The link name will still mention the word with backticks.
        let link = &link;
        let link = link.strip_prefix('`').unwrap_or(link);
        let link = link.strip_suffix('`').unwrap_or(link);

        if link_looks_like_a_path(link) {
            Some(PlainLink { name: word, link: link.to_string(), start, end })
        } else {
            None
        }
    })
}

/// Returns true if this link looks likes a valid Noir path.
fn link_looks_like_a_path(link: &str) -> bool {
    let link = link.trim();
    if link.is_empty() {
        return false;
    }
    for (index, char) in link.chars().enumerate() {
        if index == 0 {
            if !char.is_ascii_alphabetic() {
                return false;
            }
        } else if !(char.is_ascii_alphanumeric() || char == '_' || char == ':') {
            return false;
        }
    }
    true
}

/// A regex that captures markdown links as either `[reference]`, `[reference][link]` or
/// `[reference](url)`, ignoring those that happen inside backticks.
fn reference_regex() -> Regex {
    Regex::new(
        r#"(?x)
        # Ignore links inside backticks
        (?:`[^`]*`)|

        # Match [reference], [reference][link] or [reference](url)
        \[([^\[\]]+)\](?:\[([^\[\]]*)\]|\(([^\(\)]*)\))?
    "#,
    )
    .unwrap()
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
    let segments: Vec<&str> = path.split("::").collect();

    if let Some(current_type) = current_type {
        if segments.len() <= 2 && segments[0] == "Self" {
            let method_name = segments.get(1).copied();
            match current_type {
                CurrentType::Type(type_id) => {
                    if let Some(method_name) = method_name {
                        return type_method_or_field_link_target(type_id, method_name, interner);
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

                    // Array and Vector need special handling because they are composite types
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
                        PrimitiveTypeKind::Vector => {
                            let typ =
                                noirc_frontend::Type::Vector(Box::new(noirc_frontend::Type::Error));
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

    let crate_id = module_id.krate;

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
        if check_dependencies && *first_segment == "super" {
            if let Some(parent_module) =
                get_parent_module(ModuleDefId::ModuleId(module_id), interner, def_maps)
            {
                segments.remove(0);
                let path = segments.join("::");
                return path_to_link_target_searching_modules(
                    &path,
                    parent_module,
                    false,
                    interner,
                    def_maps,
                    crate_graph,
                );
            }
        }
        if check_dependencies && *first_segment == "dep" {
            segments.remove(0);
            return path_to_link_target_searching_dependency(
                crate_id,
                segments,
                interner,
                def_maps,
                crate_graph,
            );
        }
    }

    let mut current_module = &def_maps[&module_id.krate][module_id.local_id];

    for (index, segment) in segments.iter().enumerate() {
        let name = Ident::new(segment.to_string(), Location::dummy());
        let per_ns = current_module.scope().find_name(&name);

        if per_ns.is_none() {
            // If we can't find the first segment we can try to search in dependencies
            if index == 0 && check_dependencies {
                return path_to_link_target_searching_dependency(
                    crate_id,
                    segments,
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
            return Some(LinkTarget::TopLevelItem(module_def_id));
        }

        // We are not at the last segment. Find a module, type or trait to continue.
        let (module_def_id, _, _) = per_ns.types?;
        match module_def_id {
            ModuleDefId::ModuleId(module_id) => {
                current_module = &def_maps[&module_id.krate][module_id.local_id];
            }
            ModuleDefId::TypeId(type_id) => {
                // This must refer to a type method, so only one segment should remain
                if index != segments.len() - 2 {
                    return None;
                }
                let method_name = segments.last().unwrap();
                return type_method_or_field_link_target(type_id, method_name, interner);
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

/// Starts the search in a dependency, assuming the first segment is the dependency name.
/// Returns `None` if there's no first segment, or if the dependency is not found.
fn path_to_link_target_searching_dependency(
    crate_id: CrateId,
    mut segments: Vec<&str>,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    crate_graph: &CrateGraph,
) -> Option<LinkTarget> {
    let dependency_name = segments.first()?;
    let crate_data = &crate_graph[crate_id];
    let dependency_crate_id = crate_data.dependencies.iter().find_map(|dependency| {
        if &dependency.as_name() == dependency_name { Some(dependency.crate_id) } else { None }
    })?;
    let dependency_local_module_id = def_maps[&dependency_crate_id].root();
    let dependency_module_id =
        ModuleId { krate: dependency_crate_id, local_id: dependency_local_module_id };
    segments.remove(0);
    let path = segments.join("::");
    path_to_link_target_searching_modules(
        &path,
        dependency_module_id,
        false,
        interner,
        def_maps,
        crate_graph,
    )
}

fn type_method_or_field_link_target(
    type_id: TypeId,
    method_name: &str,
    interner: &NodeInterner,
) -> Option<LinkTarget> {
    let data_type = interner.get_type(type_id);
    let generic_types = data_type.borrow().generic_types();
    let typ = noirc_frontend::Type::DataType(data_type.clone(), generic_types.clone());
    if let Some(methods) = interner.get_type_methods(&typ) {
        if let Some(method) = methods.get(method_name) {
            if let Some(method) = method.direct.first() {
                let method = method.method;
                return Some(LinkTarget::Method(ModuleDefId::TypeId(type_id), method));
            }
        }
    }

    if let Some((_, _, index)) = data_type.borrow().get_field(method_name, &generic_types) {
        return Some(LinkTarget::StructMember(type_id, index));
    }

    None
}

fn trait_method_link_target(
    trait_id: TraitId,
    method_name: &str,
    interner: &NodeInterner,
) -> Option<LinkTarget> {
    let trait_ = interner.get_trait(trait_id);
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

#[cfg(test)]
mod tests {
    use crate::links::{find_links_in_markdown_line, reference_regex};

    #[test]
    fn finds_reference_plain() {
        let line = "Hello [world]!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(&link.name, "world");
        assert_eq!(&link.link, "world");
        assert_eq!(link.start, 6);
        assert_eq!(link.end, 13);
    }

    #[test]
    fn finds_reference_link_brackets() {
        let line = "Hello [world][url]!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(&link.name, "world");
        assert_eq!(&link.link, "url");
        assert_eq!(link.start, 6);
        assert_eq!(link.end, 18);
    }

    #[test]
    fn finds_reference_link_parentheses() {
        let line = "Hello [world](url)!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(&link.name, "world");
        assert_eq!(&link.link, "url");
        assert_eq!(link.start, 6);
        assert_eq!(link.end, 18);
    }

    #[test]
    fn finds_reference_with_backquotes() {
        let line = "Hello [`world`]!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(&link.name, "`world`");
        assert_eq!(&link.link, "world");
        assert_eq!(link.start, 6);
        assert_eq!(link.end, 15);
    }

    #[test]
    fn does_not_find_reference_in_backquote() {
        let line = "Hello `[world]`! Code: `let x = [foo];`. Hello [world]!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(&link.name, "world");
        assert_eq!(&link.link, "world");
        assert_eq!(link.start, 47);
        assert_eq!(link.end, 54);
    }

    #[test]
    fn does_not_find_if_empty() {
        let line = "Hello []!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert!(links.is_empty());
    }

    #[test]
    fn does_not_find_if_all_spaces() {
        let line = "Hello [  ]!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert!(links.is_empty());
    }

    #[test]
    fn does_not_find_if_not_a_valid_path() {
        let line = "Hello [ 1 + 2 ]!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert!(links.is_empty());
    }

    #[test]
    fn does_not_find_if_left_bracket_is_escaped() {
        let line = "Hello \\[foo]!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert!(links.is_empty());
    }

    #[test]
    fn does_not_find_if_right_bracket_is_escaped() {
        let line = "Hello [foo\\]!";
        let links = find_links_in_markdown_line(line, &reference_regex()).collect::<Vec<_>>();
        assert!(links.is_empty());
    }
}
