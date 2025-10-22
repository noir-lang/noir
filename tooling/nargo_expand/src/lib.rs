use nargo_fmt::ImportsGranularity;
use noirc_driver::CrateId;
use noirc_frontend::{
    graph::CrateGraph,
    hir::{def_map::DefMaps, printer::display_crate},
    node_interner::NodeInterner,
    parse_program_with_dummy_file,
};

/// Returns the expanded code for the given crate.
/// This method calls out to the frontend's HIR display functionality and either formats it or adds a warning if there are syntax errors.
pub fn get_expanded_crate(
    crate_id: CrateId,
    crate_graph: &CrateGraph,
    def_maps: &DefMaps,
    interner: &NodeInterner,
) -> String {
    let mut expanded_source = display_crate(crate_id, crate_graph, def_maps, interner);
    let (parsed_module, errors) = parse_program_with_dummy_file(&expanded_source);
    if errors.is_empty() {
        let config = nargo_fmt::Config {
            reorder_imports: true,
            imports_granularity: ImportsGranularity::Crate,
            ..Default::default()
        };
        nargo_fmt::format(&expanded_source, parsed_module, &config)
    } else {
        expanded_source.push_str("\n\n// Warning: the generated code has syntax errors");
        expanded_source
    }
}
