use std::borrow::Cow;

use noirc_errors::Location;
use petgraph::{
    algo::tarjan_scc,
    graph::{DiGraph, NodeIndex as PetGraphIndex},
};

use crate::{
    Type,
    hir::{def_collector::dc_crate::CompilationError, resolution::errors::ResolverError},
    node_interner::{FuncId, GlobalId, TraitId, TypeAliasId, TypeId},
};

use super::NodeInterner;

/// A dependency in the dependency graph may be a type or a definition.
/// Types can depend on definitions too. E.g. `Foo` depends on `COUNT` in:
///
/// ```struct
/// global COUNT = 3;
///
/// struct Foo {
///     array: [Field; COUNT],
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DependencyId {
    Struct(TypeId),
    Global(GlobalId),
    Function(FuncId),
    Alias(TypeAliasId),
    Trait(TraitId),
    Variable(Location),
}

impl NodeInterner {
    /// Gets the dependency graph from the node interner.
    pub fn dependency_graph(&self) -> &DiGraph<DependencyId, ()> {
        &self.dependency_graph
    }

    /// Register that `dependent` depends on `dependency`.
    /// This is usually because `dependent` refers to `dependency` in one of its struct fields.
    pub fn add_type_dependency(&mut self, dependent: DependencyId, dependency: TypeId) {
        self.add_dependency(dependent, DependencyId::Struct(dependency));
    }

    /// Mark a [DependencyId] as being dependant on a [GlobalId].
    pub fn add_global_dependency(&mut self, dependent: DependencyId, dependency: GlobalId) {
        self.add_dependency(dependent, DependencyId::Global(dependency));
    }

    /// Mark a [DependencyId] as being dependant on a [FuncId].
    pub fn add_function_dependency(&mut self, dependent: DependencyId, dependency: FuncId) {
        self.add_dependency(dependent, DependencyId::Function(dependency));
    }

    /// Mark a [DependencyId] as being dependant on a [TypeAliasId].
    pub fn add_type_alias_dependency(&mut self, dependent: DependencyId, dependency: TypeAliasId) {
        self.add_dependency(dependent, DependencyId::Alias(dependency));
    }

    /// Mark a [DependencyId] as being dependant on a [TraitId].
    pub fn add_trait_dependency(&mut self, dependent: DependencyId, dependency: TraitId) {
        self.add_dependency(dependent, DependencyId::Trait(dependency));
    }

    pub fn add_dependency(&mut self, dependent: DependencyId, dependency: DependencyId) {
        let dependent_index = self.get_or_insert_dependency(dependent);
        let dependency_index = self.get_or_insert_dependency(dependency);
        self.dependency_graph.update_edge(dependent_index, dependency_index, ());
    }

    pub fn get_or_insert_dependency(&mut self, id: DependencyId) -> PetGraphIndex {
        if let Some(index) = self.dependency_graph_indices.get(&id) {
            return *index;
        }

        let index = self.dependency_graph.add_node(id);
        self.dependency_graph_indices.insert(id, index);
        index
    }

    pub(crate) fn check_for_dependency_cycles(&self) -> Vec<CompilationError> {
        let strongly_connected_components = tarjan_scc(&self.dependency_graph);
        let mut errors = Vec::new();

        let mut push_error = |item: String, scc: &[_], i, location: Location| {
            let cycle = self.get_cycle_error_string(scc, i);
            let error = ResolverError::DependencyCycle { item, cycle, location };
            errors.push(error.into());
        };

        for scc in strongly_connected_components {
            if scc.len() > 1 {
                // If a SCC contains a type, type alias, or global, it must be the only element in the SCC
                for (i, index) in scc.iter().enumerate() {
                    match self.dependency_graph[*index] {
                        DependencyId::Struct(struct_id) => {
                            let struct_type = self.get_type(struct_id);
                            let struct_type = struct_type.borrow();
                            push_error(struct_type.name.to_string(), &scc, i, struct_type.location);
                            break;
                        }
                        DependencyId::Global(global_id) => {
                            let global = self.get_global(global_id);
                            let name = global.ident.to_string();
                            push_error(name, &scc, i, global.location);
                            break;
                        }
                        DependencyId::Alias(alias_id) => {
                            let alias = self.get_type_alias(alias_id);
                            // If type aliases form a cycle, we have to manually break the cycle
                            // here to prevent infinite recursion in the type checker.
                            alias.borrow_mut().typ = Type::Error;

                            // push_error will borrow the alias so we have to drop the mutable borrow
                            let alias = alias.borrow();
                            push_error(alias.name.to_string(), &scc, i, alias.location);
                            break;
                        }
                        DependencyId::Trait(trait_id) => {
                            let the_trait = self.get_trait(trait_id);
                            push_error(the_trait.name.to_string(), &scc, i, the_trait.location);
                            break;
                        }
                        // Mutually recursive functions are allowed
                        DependencyId::Function(_) => (),
                        // Local variables should never be in a dependency cycle, scoping rules
                        // prevents referring to them before they're defined
                        DependencyId::Variable(loc) => unreachable!(
                            "Variable used at location {loc:?} caught in a dependency cycle"
                        ),
                    }
                }
            }
        }

        errors
    }

    /// Build up a string starting from the given item containing each item in the dependency
    /// cycle. The final result will resemble `foo -> bar -> baz -> foo`, always going back to the
    /// element at the given start index.
    fn get_cycle_error_string(&self, scc: &[PetGraphIndex], start_index: usize) -> String {
        let index_to_string = |index: PetGraphIndex| match self.dependency_graph[index] {
            DependencyId::Struct(id) => Cow::Owned(self.get_type(id).borrow().name.to_string()),
            DependencyId::Function(id) => Cow::Borrowed(self.function_name(&id)),
            DependencyId::Alias(id) => {
                Cow::Owned(self.get_type_alias(id).borrow().name.to_string())
            }
            DependencyId::Global(id) => Cow::Borrowed(self.get_global(id).ident.as_str()),
            DependencyId::Trait(id) => Cow::Owned(self.get_trait(id).name.to_string()),
            DependencyId::Variable(loc) => {
                unreachable!("Variable used at location {loc:?} caught in a dependency cycle")
            }
        };

        let mut cycle = index_to_string(scc[start_index]).to_string();

        // Reversing the dependencies here matches the order users would expect for the error message
        for i in (0..scc.len()).rev() {
            cycle += " -> ";
            cycle += &index_to_string(scc[(start_index + i) % scc.len()]);
        }

        cycle
    }
}
