use std::collections::{BTreeMap, BTreeSet};

use fxhash::{FxHashMap as HashMap, FxHashSet};

use crate::ssa::ir::{dfg::DataFlowGraph, types::Type, value::ValueId};

use petgraph::{Direction, prelude::NodeIndex};

use super::alias_set::AliasSet;

/// An AliasGraph represents the data flow of references in a program.
///
/// Each reference in the program may correspond to a node in this graph:
/// - If it is not in this graph, its aliases are unknown. Storing to the
///   reference may invalidate any reference in the program of the same type.
/// - If it is in the graph, it is linked to its potential aliases via edges
///   in the graph. These edges are directed in the direction of data flow in
///   the program. For example, in `v3 = if v0 then v1 else v2`, where all values
///   except `v0` are references, we would expect the edges `v1 -> v3` and `v2 -> v3`
///   indicating that `v3` may be constructed from `v1` or `v2`.
///   - In the case the direction of data flow is unknown (reference function parameters),
///     these references will have edges in both directions. For example, if the entry
///     block to our function looks like `b0(v0: &mut Field, v1: &mut Field)` then we
///     would expect both `v0 -> v1` and `v1 -> v0`. Put in another way: `v0 <-> v1`.
///
/// Each node in the graph has the value stored within that reference if known. Note
/// that because this is stored per-node we may know this value for some potential
/// aliases of a reference but not others.
///
/// Several references may correspond to the same node in the graph. This happens when
/// a reference is known to always refer to only one other reference. For example, in:
/// `v2 = if v0 then v1 else v1` the reference `v2` is known to always equal `v1` and
/// only `v1` so it shares a node with `v1`. This property isn't necessary for correctness
/// but is instead an optimization that ensures when `v1` is stored to, `v2`'s value becomes
/// known and vice-versa.
#[derive(Debug, Default, Clone)]
pub(super) struct AliasGraph {
    graph: InnerGraph,

    /// Maps an address (a ValueId of type `Type::Reference(..)`) to its index in the node
    /// graph. Note that there are only references in the graph, there are no arrays of
    /// references for example. Nested references include only their outer aliases, for
    /// tracking which values they may contain, we need to track their Container separately,
    /// as is done for arrays of references.
    address_to_node: BTreeMap<ValueId, NodeIndex>,

    /// Maps a reference type to all addresses of that type. When a reference's aliases are
    /// otherwise unknown it is given every reference of a matching type as an alias.
    type_to_node: HashMap<Type, FxHashSet<ValueId>>,

    /// Maps node indices to the address(es) it represents. Used for debugging.
    node_to_address: HashMap<NodeIndex, BTreeSet<ValueId>>,
}

type InnerGraph = petgraph::prelude::DiGraph<ReferenceValue, ()>;

/// The result of loading a particular reference.
/// This is `None` when unknown, otherwise it is `Some(known_value)`.
pub(super) type ReferenceValue = Option<ValueId>;

impl AliasGraph {
    /// Invalidates a given reference's value.
    ///
    /// To invalidate a reference we:
    /// - Check if `reference` is in the graph (ie. its aliases are known)
    ///   - If it is not, mark all reference values unknown and stop. (this could be narrowed to only references of
    ///   the same type)
    ///   - Otherwise, iterate each neighbor of the node `reference` corresponds to regardless of
    ///   the direction of the edge to that neighbor. For each neighbor:
    ///     - Iterate through all nodes reachable from that neighbor, setting all of their
    ///     ReferenceValues to None. When finding reachable nodes, edges should only be followed
    ///     in their correct direction (ie. from `a`, follow `a -> b` but not `a <- c`).
    pub(crate) fn invalidate(&mut self, reference: ValueId) {
        let original_node = *self
            .address_to_node
            .get(&reference)
            .unwrap_or_else(|| unreachable!("Node {reference} is not in alias graph!"));

        Self::traverse_possible_aliases_mut(&mut self.graph, original_node, |graph, child| {
            graph[child] = None;
        });
    }

    /// Creates a fresh reference node in the graph, aliased to nothing.
    /// This is used when a new reference is created e.g. from a `allocate` instruction.
    pub(crate) fn new_reference(&mut self, reference: ValueId, reference_type: Type) -> NodeIndex {
        let index = self.new_orphan_reference();
        self.address_to_node.insert(reference, index);
        self.node_to_address.entry(index).or_default().insert(reference);
        self.type_to_node.entry(reference_type).or_default().insert(reference);
        index
    }

    /// Creates a fresh reference node in the graph, aliased to nothing.
    /// Unlike `new_reference`, this only creates the node in the graph and does not connect
    /// it to any actual value ids. This can be used for representing references outside the
    /// current function for which there are no value ids.
    #[inline(always)]
    pub(crate) fn new_orphan_reference(&mut self) -> NodeIndex {
        self.graph.add_node(None)
    }

    pub(crate) fn add_directed_alias(&mut self, from: ValueId, to: ValueId) {
        let from = self.address_to_node.get(&from).copied().expect("`from` must be in graph");
        let to = self.address_to_node.get(&to).copied().expect("`to` must be in graph");
        self.add_directed_alias_from_indices(from, to);
    }

    pub(crate) fn add_directed_alias_from_indices(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.update_edge(from, to, ());
    }

    /// Adds a bidirectional alias between two existing nodes in the graph.
    /// This is used when the direction of data flow is unknown, for example
    /// between function parameters.
    pub(crate) fn add_undirected_alias_from_indices(&mut self, from: NodeIndex, to: NodeIndex) {
        self.add_directed_alias_from_indices(from, to);
        self.add_directed_alias_from_indices(to, from);
    }

    /// Creates a new reference from the given pre-existing reference(s).
    /// This is used when a reference value is created which must be equal to a
    /// pre-existing reference. Examples include `v2 = if .. then v0 else v1`,
    /// or `jmp b0(v0) ... jmp b0(v1) ... b0(v2: &mut Field):` where `v0` and `v1`
    /// are the `derived_from` references and `v2` is the `new_reference` created from them.
    ///
    /// Because of the optimization to set this reference equal to a derived one if only
    /// a single derived reference was given, this method should be avoided unless
    /// `derived_from` includes all references this was derived from. If you want to build
    /// a reference adding one alias at a time, use `new_reference` and `add_alias` separately.
    pub(crate) fn new_derived_reference(
        &mut self,
        new_reference: ValueId,
        derived_from: &[ValueId],
        reference_type: Type,
    ) {
        // Optimization: if there is only one reference we may be derived from, just
        // re-use the same node for that reference so they directly share ReferenceValues.
        if derived_from.len() == 1 {
            let node = self.address_to_node[&derived_from[0]];
            self.address_to_node.insert(new_reference, node);
        } else {
            let new_reference = self.new_reference(new_reference, reference_type);
            for child in derived_from {
                let child = self.address_to_node[child];
                // data flows from the child to the new reference
                self.graph.update_edge(child, new_reference, ());
            }
        }
    }

    pub(crate) fn new_derived_reference_from_alias_set(
        &mut self,
        new_reference: ValueId,
        aliases: &AliasSet,
        reference_type: Type,
    ) {
        let aliases = if aliases.is_unknown() {
            // If the aliases are unknown we have to link it to every value id of the given type
            self.type_to_node[&reference_type].iter().copied().collect::<Vec<_>>()
        } else {
            aliases.iter().collect::<Vec<_>>()
        };
        self.new_derived_reference(new_reference, &aliases, reference_type);
    }

    /// Invalidate the values of this references aliases then set the value
    /// of this reference to the given value.
    pub(crate) fn store_to_reference(&mut self, reference: ValueId, value: ValueId) {
        self.invalidate(reference);

        if let Some(node) = self.address_to_node.get(&reference) {
            self.graph[*node] = Some(value);
        }
    }

    /// Return the value stored at `reference`, if known.
    pub(crate) fn get_known_value(&self, reference: ValueId) -> Option<ValueId> {
        let node = self.address_to_node.get(&reference)?;
        self.graph[*node]
    }

    fn traverse_possible_aliases(
        graph: &InnerGraph,
        node: NodeIndex,
        mut f: impl FnMut(&InnerGraph, NodeIndex),
    ) {
        // The reference is always an alias to itself
        f(graph, node);

        let mut neighbors = graph.neighbors_undirected(node).detach();
        while let Some((_, neighbor)) = neighbors.next(&graph) {
            // Perform a dfs from each neighbor, performing `f` on each
            let mut dfs = petgraph::visit::Dfs::new(&*graph, neighbor);
            while let Some(node) = dfs.next(&*graph) {
                f(graph, node);
            }
        }
    }

    fn traverse_possible_aliases_mut(
        graph: &mut InnerGraph,
        node: NodeIndex,
        mut f: impl FnMut(&mut InnerGraph, NodeIndex),
    ) {
        // The reference is always an alias to itself
        f(graph, node);

        let mut neighbors = graph.neighbors_undirected(node).detach();
        while let Some((_, neighbor)) = neighbors.next(&graph) {
            // Perform a dfs from each neighbor, performing `f` on each
            let mut dfs = petgraph::visit::Dfs::new(&*graph, neighbor);
            while let Some(node) = dfs.next(&*graph) {
                f(graph, node);
            }
        }
    }

    /// Returns the possible aliases of the given reference if known.
    /// If unknown, None is returned.
    ///
    /// This is the same traversal used by `invalidate`
    pub(crate) fn possible_aliases(&self, reference: ValueId) -> AliasSet {
        let Some(node) = self.address_to_node.get(&reference).copied() else {
            return AliasSet::unknown();
        };

        let mut aliases = AliasSet::known_empty();
        Self::traverse_possible_aliases(&self.graph, node, |_, alias| {
            if let Some(more_aliases) = self.node_to_address.get(&alias) {
                aliases.unify(&AliasSet::known_multiple(more_aliases.clone().into()));
            } else {
                aliases = AliasSet::unknown();
            }
        });
        aliases
    }

    /// Merge two graphs: `G1 = (N1, E1)` and `G2 = (N2, E2)`.
    ///
    /// A merged graph has nodes `N1 ∪ N2` and edges `E1 ∪ E2`.
    /// The value for each node `N` will be:
    /// - `V` iff `(*N = V) ∈ N1 && (*N = V) ∈ N2`
    /// - `None` otherwise.
    pub(crate) fn unify(&self, other: &Self, dfg: &DataFlowGraph) -> Self {
        // We're going to union most of the graph so start out with the `self` graph so
        // we only have to add in `other`. This also means indices in `new_graph` will match
        // those in `self` except for new nodes/edges added from `other`.
        let mut new_graph = self.graph.clone();
        let mut address_to_node = self.address_to_node.clone();
        let mut node_to_address = self.node_to_address.clone();
        let mut type_to_node = self.type_to_node.clone();

        for node_value in new_graph.node_weights_mut() {
            *node_value = None;
        }

        // Merge the two `address_to_node` maps, inserting shared nodes into our graph
        for (address, other_index) in other.address_to_node.iter() {
            // The node exists in both graphs, we can set its value to the union of both
            if let Some(self_index) = self.address_to_node.get(address) {
                let self_value = self.graph[*self_index];
                let other_value = other.graph[*other_index];
                let merged_value = if self_value == other_value { self_value } else { None };
                new_graph[*self_index] = merged_value;
            } else {
                // The node is only in `other`, ensure it is included in this graph but
                // its value is None.
                let node = new_graph.add_node(None);
                address_to_node.insert(*address, node);
                node_to_address.entry(node).or_default().insert(*address);

                let address_type = dfg.type_of_value(*address);
                type_to_node.entry(address_type).or_default().insert(*address);
            }
        }

        // For each node, unify each edge from both directions
        for edge in other.graph.edge_indices() {
            let (start, end) = other.graph.edge_endpoints(edge).expect("Edge should be in graph");
            new_graph.update_edge(start, end, ());
        }

        Self { graph: new_graph, address_to_node, node_to_address, type_to_node }
    }

    pub(crate) fn has_node(&self, parameter: ValueId) -> bool {
        self.address_to_node.contains_key(&parameter)
    }
}

// Show a representation of this graph for debugging purposes
impl std::fmt::Display for AliasGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (address, node) in self.address_to_node.iter() {
            let address_string = address.to_string();
            let address_string_len = address_string.len();
            write!(f, "{address_string}")?;

            let mut print_neighbors = |direction, direction_arrow, print_on_newline| {
                let mut neighbors = self.graph.neighbors_directed(*node, direction).peekable();
                let has_neighbors = neighbors.peek().is_some();
                if has_neighbors {
                    if print_on_newline {
                        let spaces = " ".repeat(address_string_len);
                        write!(f, "\n{spaces}")?;
                    }
                    write!(f, "{}", direction_arrow)?;
                }

                for (i, node) in neighbors.enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    let aliases = &self.node_to_address[&node];
                    write_alias_set_string(aliases, f)?;
                }
                Ok::<_, std::fmt::Error>(has_neighbors)
            };

            // Incoming neighbors are printed first since they're more important.
            // These represent the values this value may have been constructed from.
            let has_incoming = print_neighbors(Direction::Incoming, " <- ", false)?;
            print_neighbors(Direction::Outgoing, " -> ", has_incoming)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

fn write_alias_set_string(
    set: &BTreeSet<ValueId>,
    f: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    if set.len() == 1 {
        return write!(f, "{}", set.first().unwrap());
    }

    write!(f, "{{")?;
    for (i, address) in set.iter().enumerate() {
        if i != 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}", address)?;
    }
    write!(f, "}}")
}

#[cfg(test)]
mod tests {
    use crate::ssa::{
        ir::basic_block::BasicBlockId,
        opt::{
            assert_normalized_ssa_equals,
            mem2reg::{PerFunctionContext, alias_graph::AliasGraph},
            normalize_value_ids,
        },
        ssa_gen::Ssa,
    };

    /// Returns the alias graph after the given block, after running mem2reg on the given SSA source.
    fn alias_graph_after_block(ssa_src: &str, block: Option<BasicBlockId>) -> AliasGraph {
        let mut ssa = Ssa::from_str(ssa_src).unwrap().mem2reg();
        assert_eq!(ssa.functions.len(), 1);

        let main = ssa.functions.values_mut().next().unwrap();
        let mut context = PerFunctionContext::new(main);
        context.mem2reg();

        // mem2reg messes up the ValueIds even if we normalize right before it. So
        // we have to normalize after and apply those mappings to the alias graph
        // so that it matches the normalized Ssa instead of an implementation-defined one.
        let block = block.unwrap_or_else(|| context.inserter.function.find_last_block());
        let mut graph = context.blocks[&block].alias_graph.clone();
        let ids = normalize_value_ids::normalize_single_function(&mut ssa);
        assert_normalized_ssa_equals(ssa, ssa_src);

        graph.address_to_node = graph
            .address_to_node
            .into_iter()
            .map(|(address, node)| (ids.values[&address], node))
            .collect();

        for addresses in graph.node_to_address.values_mut() {
            *addresses = addresses.iter().map(|address| ids.values[address]).collect();
        }

        graph
    }

    fn alias_graph_after_last_block(ssa: &str) -> AliasGraph {
        alias_graph_after_block(ssa, None)
    }

    #[test]
    fn create_note() {
        let ssa = "
            acir(inline) fn create_note f0 {
              b0(v0: &mut Field, v1: &mut Field, v2: u1):
                jmpif v2 then: b1, else: b2
              b1():
                jmp b3(v0, v1)
              b2():
                v6 = allocate -> &mut Field
                store Field 0 at v6
                jmp b3(v0, v6)
              b3(v3: &mut Field, v4: &mut Field):
                store Field 2 at v3
                store Field 3 at v4
                v5 = load v3 -> Field
                return v5
            }";

        // Limitation:
        //  v6 is unknown in b1 so when it is merged into b3 it remains
        //  unknown, causing v4 to be unknown as well.
        let expected = "v0 <- v1
   -> v3, v1
v1 <- v0
   -> v0
v3 <- v0
";

        let graph = alias_graph_after_last_block(ssa);
        println!("{graph}");
        assert_eq!(graph.to_string(), expected);
    }

    #[test]
    fn keep_repeat_loads_with_alias_store_nested_alias_graph() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b2, else: b1
          b1():
            v8 = allocate -> &mut Field
            store Field 1 at v8
            v10 = allocate -> &mut Field
            store Field 2 at v10
            v12 = allocate -> &mut &mut Field
            store v10 at v12
            jmp b3(v8, v8, v12)
          b2():
            v4 = allocate -> &mut Field
            store Field 0 at v4
            v6 = allocate -> &mut Field
            v7 = allocate -> &mut &mut Field
            store v4 at v7
            jmp b3(v4, v4, v7)
          b3(v1: &mut Field, v2: &mut Field, v3: &mut &mut Field):
            v13 = load v1 -> Field
            store Field 2 at v2
            v14 = load v1 -> Field
            store Field 1 at v2
            v15 = load v1 -> Field
            store Field 3 at v2
            v17 = load v1 -> Field
            constrain v13 == Field 0
            constrain v14 == Field 2
            constrain v15 == Field 1
            constrain v17 == Field 3
            v18 = load v3 -> &mut Field
            v19 = load v18 -> Field
            store Field 5 at v2
            v21 = load v3 -> &mut Field
            v22 = load v21 -> Field
            constrain v19 == Field 3
            constrain v22 == Field 5
            return
        }
        ";
        let graph = alias_graph_after_last_block(src);
        println!("{graph}");
        assert_eq!(graph.to_string(), "");
    }
}
