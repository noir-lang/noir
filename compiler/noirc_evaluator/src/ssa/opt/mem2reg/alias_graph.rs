use std::collections::{BTreeMap, BTreeSet};

use fxhash::FxHashMap as HashMap;

use crate::ssa::ir::value::ValueId;

use petgraph::{prelude::NodeIndex, visit::IntoNodeIdentifiers, Direction};

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

    /// Maps node indices to the address(es) it represents.
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
    pub fn invalidate(&mut self, reference: ValueId) {
        let Some(original_node) = self.address_to_node.get(&reference).copied() else {
            // Aliases are unknown
            self.mark_all_unknown();
            return;
        };

        Self::traverse_possible_aliases_mut(&mut self.graph, original_node, |graph, child| {
            graph[child] = None;
        });
    }

    /// Creates a fresh reference node in the graph, aliased to nothing.
    /// This is used when a new reference is created e.g. from a `allocate` instruction.
    pub fn new_reference(&mut self, reference: ValueId) -> NodeIndex {
        let index = self.new_orphan_reference();
        self.address_to_node.insert(reference, index);
        self.node_to_address.entry(index).or_default().insert(reference);
        index
    }

    /// Creates a fresh reference node in the graph, aliased to nothing.
    /// Unlike `new_reference`, this only creates the node in the graph and does not connect
    /// it to any actual value ids. This can be used for representing references outside the
    /// current function for which there are no value ids.
    #[inline(always)]
    pub fn new_orphan_reference(&mut self) -> NodeIndex {
        self.graph.add_node(None)
    }

    pub fn add_directed_alias(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.update_edge(from, to, ());
    }

    pub fn add_undirected_alias(&mut self, from: NodeIndex, to: NodeIndex) {
        self.add_directed_alias(from, to);
        self.add_directed_alias(to, from);
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
    pub fn new_derived_reference(&mut self, new_reference: ValueId, derived_from: &[ValueId]) {
        // Optimization: if there is only one reference we may be derived from, just
        // re-use the same node for that reference so they directly share ReferenceValues.
        if derived_from.len() == 1 {
            if let Some(node) = self.address_to_node.get(&derived_from[0]).copied() {
                self.address_to_node.insert(new_reference, node);
            }
            // otherwise, derived_from[0] is unknown, so new_refernce should be too - don't insert it.
        } else {
            let derived_from = derived_from.iter().map(|child| {
                self.address_to_node.get(child).copied()
            }).collect::<Option<Vec<_>>>();

            if let Some(derived_from) = derived_from {
                // All values are known
                let new_reference = self.new_reference(new_reference);
                for child in derived_from {
                    // data flows from the child to the new reference
                    self.graph.update_edge(child, new_reference, ());
                }
            }
            // otherwise, if at least 1 value is unknown, so is `new_reference` - don't insert it.
        }
    }

    /// Invalidate the values of this references aliases then set the value
    /// of this reference to the given value.
    pub fn store_to_reference(&mut self, reference: ValueId, value: ValueId) {
        self.invalidate(reference);

        if let Some(node) = self.address_to_node.get(&reference) {
            self.graph[*node] = Some(value);
        }
    }

    /// Marks the value of all references as unknown. This does not affect edges in the graph
    /// (possible aliases between references).
    pub fn mark_all_unknown(&mut self) {
        for node in self.graph.node_identifiers() {
            self.graph[node] = None;
        }
    }

    /// Return the value stored at `reference`, if known.
    pub fn get_known_value(&self, reference: ValueId) -> Option<ValueId> {
        let node = self.address_to_node.get(&reference)?;
        self.graph[*node]
    }

    fn traverse_possible_aliases(graph: &InnerGraph, node: NodeIndex, mut f: impl FnMut(&InnerGraph, NodeIndex)) {
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

    fn traverse_possible_aliases_mut(graph: &mut InnerGraph, node: NodeIndex, mut f: impl FnMut(&mut InnerGraph, NodeIndex)) {
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
    pub fn possible_aliases(&self, reference: ValueId) -> AliasSet {
        let Some(node) = self.address_to_node.get(&reference).copied() else {
            return AliasSet::unknown();
        };

        let mut aliases = AliasSet::known_empty();
        Self::traverse_possible_aliases(&self.graph, node, |_, alias| {
            let more_aliases = &self.node_to_address[&node];
            aliases.unify(&AliasSet::known_multiple(more_aliases.clone().into()));
        });
        aliases
    }

    pub fn unify(&self, other: &Self) -> Self {
        todo!("AliasGraph::unify")
    }
}

// Show a representation of this graph for debugging purposes
impl std::fmt::Display for AliasGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (address, node) in self.address_to_node.iter() {
            let address_string = address.to_string();
            let address_string_len = address_string.len();
            write!(f, "{address_string}");

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
        }
        Ok(())
    }
}

fn write_alias_set_string(set: &BTreeSet<ValueId>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{{")?;
    for (i, address) in set.iter().enumerate() {
        if i != 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}", address)?;
    }
    write!(f, "}}")
}
