use std::collections::BTreeMap;

use nargo_fmt::ImportsGranularity;
use noirc_driver::CrateId;
use noirc_frontend::{
    graph::CrateGraph,
    hir::def_map::{CrateDefMap, ModuleId},
    node_interner::NodeInterner,
    parse_program_with_dummy_file,
};

use crate::{items::ItemBuilder, printer::ItemPrinter};

mod items;
mod printer;

/// Returns the expanded code for the given crate.
pub fn get_expanded_crate(
    crate_id: CrateId,
    crate_graph: &CrateGraph,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    interner: &NodeInterner,
) -> String {
    let root_module_id = def_maps[&crate_id].root();
    let module_id = ModuleId { krate: crate_id, local_id: root_module_id };

    let mut builder = ItemBuilder::new(crate_id, interner, def_maps);
    let item = builder.build_module(module_id);

    let dependencies = &crate_graph[crate_id].dependencies;

    let mut string = String::new();
    let mut printer = ItemPrinter::new(crate_id, interner, def_maps, dependencies, &mut string);
    printer.show_item(item);

    let (parsed_module, errors) = parse_program_with_dummy_file(&string);
    if errors.is_empty() {
        let config = nargo_fmt::Config {
            reorder_imports: true,
            imports_granularity: ImportsGranularity::Crate,
            ..Default::default()
        };
        nargo_fmt::format(&string, parsed_module, &config)
    } else {
        string.push_str("\n\n// Warning: the generated code has syntax errors");
        string
    }
}
