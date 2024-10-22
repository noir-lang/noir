use fm::FileId;
use noirc_errors::Span;
use petgraph::prelude::{DiGraph, NodeIndex};
use rustc_hash::FxHashMap as HashMap;

use crate::{
    ast::Expression,
    hir::{comptime::Value, def_map::LocalModuleId},
    node_interner::FuncId,
};

#[derive(Debug)]
pub struct AttributeGraph {
    default_stage: NodeIndex,

    order: DiGraph<FuncId, f32>,

    indices: HashMap<FuncId, NodeIndex>,

    modified_functions: std::collections::HashSet<FuncId>,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct AttributeContext {
    // The file where generated items should be added
    pub(crate) file: FileId,
    // The module where generated items should be added
    pub(crate) module: LocalModuleId,
    // The file where the attribute is located
    pub(crate) attribute_file: FileId,
    // The module where the attribute is located
    pub(crate) attribute_module: LocalModuleId,
}

pub(crate) type CollectedAttributes = Vec<(FuncId, Value, Vec<Expression>, AttributeContext, Span)>;

impl AttributeContext {
    pub(crate) fn new(file: FileId, module: LocalModuleId) -> Self {
        Self { file, module, attribute_file: file, attribute_module: module }
    }
}

impl Default for AttributeGraph {
    fn default() -> Self {
        let mut order = DiGraph::default();
        let mut indices = HashMap::default();

        let default_stage = order.add_node(FuncId::dummy_id());
        indices.insert(FuncId::dummy_id(), default_stage);

        Self { default_stage, order, indices, modified_functions: Default::default() }
    }
}

impl AttributeGraph {
    pub fn get_or_insert(&mut self, attr: FuncId) -> NodeIndex {
        if let Some(index) = self.indices.get(&attr) {
            return *index;
        }

        let index = self.order.add_node(attr);
        self.indices.insert(attr, index);
        index
    }

    pub fn add_ordering_constraint(&mut self, run_first: FuncId, run_second: FuncId) {
        let first_index = self.get_or_insert(run_first);
        let second_index = self.get_or_insert(run_second);

        // Just for debugging
        if run_first != FuncId::dummy_id() {
            self.modified_functions.insert(run_first);
            self.modified_functions.insert(run_second);
        }

        self.order.update_edge(second_index, first_index, 1.0);
    }

    /// The default ordering of an attribute: run in the default stage
    pub fn run_in_default_stage(&mut self, attr: FuncId) {
        let index = self.get_or_insert(attr);
        self.order.update_edge(self.default_stage, index, 1.0);
    }

    pub(crate) fn sort_attributes_by_run_order(&self, attributes: &mut CollectedAttributes) {
        let topological_sort = petgraph::algo::toposort(&self.order, None).unwrap();

        let ordering: HashMap<FuncId, usize> =
            topological_sort.into_iter().map(|index| (self.order[index], index.index())).collect();

        attributes.sort_by_key(|(f, ..)| ordering[f]);
    }
}
