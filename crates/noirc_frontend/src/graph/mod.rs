// This has been taken and modified from the rust-analyzer codebase
// For the most part, everything is the same, the differences are quite subtle
// but still present. Moreover, since RA is uses incremental compilation, the usage of this component may differ.
// This version is also simpler due to not having macro_defs or proc_macros
// XXX: Edition may be reintroduced or some sort of versioning

use fm::FileId;
use rustc_hash::{FxHashMap, FxHashSet};
use smol_str::SmolStr;

/// The local crate is the crate being compiled.
/// The caller should ensure that this crate has a CrateId(0).
pub const LOCAL_CRATE: CrateId = CrateId(0);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CrateId(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateName(SmolStr);

impl CrateName {
    /// Creates a new CrateName rejecting any crate name that
    /// has a character on the blacklist.
    /// The difference between RA and this implementation is that
    /// characters on the blacklist are never allowed; there is no normalisation.
    pub fn new(name: &str) -> Result<CrateName, &str> {
        let is_invalid = name.chars().any(|n| CHARACTER_BLACK_LIST.contains(&n));
        if is_invalid {
            Err(name)
        } else {
            Ok(Self(SmolStr::new(name)))
        }
    }

    pub fn as_string(&self) -> String {
        self.0.clone().into()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrateGraph {
    arena: FxHashMap<CrateId, CrateData>,
}

/// List of characters that are not allowed in a crate name
/// For example, Hyphen(-) is disallowed as it is similar to underscore(_)
/// and we do not want names that differ by a hyphen
pub const CHARACTER_BLACK_LIST: [char; 1] = ['-'];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CrateType {
    Library,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateData {
    pub root_file_id: FileId,
    pub crate_type: CrateType,
    pub dependencies: Vec<Dependency>,
}

/// A dependency is a crate name and a crate_id
/// This means that the same crate can be compiled once under different names
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub crate_id: CrateId,
    pub name: CrateName,
}

impl Dependency {
    pub fn as_name(&self) -> String {
        self.name.as_string()
    }
}

impl CrateGraph {
    pub fn add_crate_root(&mut self, crate_type: CrateType, file_id: FileId) -> CrateId {
        let roots_with_file_id: Vec<_> = self
            .arena
            .iter()
            .filter(|(_, crate_data)| crate_data.root_file_id == file_id)
            .collect();
        assert!(
            roots_with_file_id.is_empty(),
            "you cannot add the same file id twice"
        );

        let data = CrateData {
            root_file_id: file_id,
            crate_type,
            dependencies: Vec::new(),
        };
        let crate_id = CrateId(self.arena.len());
        let prev = self.arena.insert(crate_id, data);
        assert!(prev.is_none());
        crate_id
    }

    pub fn crate_type(&self, crate_id: CrateId) -> CrateType {
        self.arena.get(&crate_id).unwrap().crate_type
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = CrateId> + '_ {
        self.arena.keys().copied()
    }

    pub fn crates_in_topological_order(&self) -> Vec<CrateId> {
        let mut res = Vec::new();
        let mut visited = FxHashSet::default();

        for krate in self.arena.keys().copied() {
            go(self, &mut visited, &mut res, krate);
        }

        return res;

        fn go(
            graph: &CrateGraph,
            visited: &mut FxHashSet<CrateId>,
            res: &mut Vec<CrateId>,
            source: CrateId,
        ) {
            if !visited.insert(source) {
                return;
            }
            for dep in graph[source].dependencies.iter() {
                go(graph, visited, res, dep.crate_id)
            }
            res.push(source)
        }
    }

    pub fn add_dep(
        &mut self,
        from: CrateId,
        name: CrateName,
        to: CrateId,
    ) -> Result<(), CyclicDependenciesError> {
        if self.dfs_find(from, to, &mut FxHashSet::default()) {
            return Err(CyclicDependenciesError { from, to });
        }
        self.arena.get_mut(&from).unwrap().add_dep(name, to);
        Ok(())
    }

    fn dfs_find(&self, target: CrateId, from: CrateId, visited: &mut FxHashSet<CrateId>) -> bool {
        if !visited.insert(from) {
            return false;
        }

        if target == from {
            return true;
        }

        for dep in &self[from].dependencies {
            let crate_id = dep.crate_id;
            if self.dfs_find(target, crate_id, visited) {
                return true;
            }
        }
        false
    }

    pub fn number_of_crates(&self) -> usize {
        self.arena.len()
    }
}
impl CrateData {
    fn add_dep(&mut self, name: CrateName, crate_id: CrateId) {
        self.dependencies.push(Dependency { crate_id, name })
    }
}
impl std::ops::Index<CrateId> for CrateGraph {
    type Output = CrateData;
    fn index(&self, crate_id: CrateId) -> &CrateData {
        &self.arena[&crate_id]
    }
}

/// XXX: This is barebone for two reasons:
// There are no display names currently
// The error would be better if it showed the full cyclic dependency, including transitives.
#[derive(Debug)]
pub struct CyclicDependenciesError {
    from: CrateId,
    to: CrateId,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CrateGraph, CrateName, CrateType, FileId};

    fn dummy_file_ids(n: usize) -> Vec<FileId> {
        use fm::{FileMap, FILE_EXTENSION};
        let mut fm = FileMap::new();

        let mut vec_ids = Vec::with_capacity(n);
        for i in 0..n {
            let mut pth = PathBuf::new();
            pth.push(format!("{}", i));
            pth.set_extension(FILE_EXTENSION);
            vec_ids.push(fm.add_file(pth.into(), String::new()));
        }

        vec_ids
    }

    #[test]
    fn detect_cyclic_dependency_indirect() {
        let file_ids = dummy_file_ids(3);

        let mut graph = CrateGraph::default();
        let crate1 = graph.add_crate_root(CrateType::Library, file_ids[0]);
        let crate2 = graph.add_crate_root(CrateType::Library, file_ids[1]);
        let crate3 = graph.add_crate_root(CrateType::Library, file_ids[2]);

        assert!(graph
            .add_dep(crate1, CrateName::new("crate2").unwrap(), crate2)
            .is_ok());
        assert!(graph
            .add_dep(crate2, CrateName::new("crate3").unwrap(), crate3)
            .is_ok());
        assert!(graph
            .add_dep(crate3, CrateName::new("crate1").unwrap(), crate1)
            .is_err());
    }

    #[test]
    fn it_works() {
        let file_ids = dummy_file_ids(3);
        let file_id_0 = file_ids[0];
        let file_id_1 = file_ids[1];
        let file_id_2 = file_ids[2];
        let mut graph = CrateGraph::default();
        let crate1 = graph.add_crate_root(CrateType::Library, file_id_0);
        let crate2 = graph.add_crate_root(CrateType::Library, file_id_1);
        let crate3 = graph.add_crate_root(CrateType::Library, file_id_2);
        assert!(graph
            .add_dep(crate1, CrateName::new("crate2").unwrap(), crate2)
            .is_ok());
        assert!(graph
            .add_dep(crate2, CrateName::new("crate3").unwrap(), crate3)
            .is_ok());
    }
    #[test]
    #[should_panic]
    fn it_works2() {
        let file_ids = dummy_file_ids(3);
        let file_id_0 = file_ids[0];
        let file_id_1 = file_ids[1];
        let file_id_2 = file_ids[2];
        let mut graph = CrateGraph::default();
        let _crate1 = graph.add_crate_root(CrateType::Library, file_id_0);
        let _crate2 = graph.add_crate_root(CrateType::Library, file_id_1);
        let _crate3 = graph.add_crate_root(CrateType::Library, file_id_2);
        let _crate3 = graph.add_crate_root(CrateType::Library, file_id_2);
    }
}
