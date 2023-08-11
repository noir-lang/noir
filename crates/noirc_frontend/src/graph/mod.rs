// This has been taken and modified from the rust-analyzer codebase
// For the most part, everything is the same, the differences are quite subtle
// but still present. Moreover, since RA is uses incremental compilation, the usage of this component may differ.
// This version is also simpler due to not having macro_defs or proc_macros
// XXX: Edition may be reintroduced or some sort of versioning

use std::{fmt::Display, option::Option, str::FromStr};

use fm::FileId;
use rustc_hash::{FxHashMap, FxHashSet};
use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrateId {
    Crate(usize),
    Stdlib(usize),
}

impl CrateId {
    pub fn dummy_id() -> CrateId {
        CrateId::Crate(std::usize::MAX)
    }

    pub fn is_stdlib(&self) -> bool {
        matches!(self, CrateId::Stdlib(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CrateName(SmolStr);

impl Display for CrateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<CrateName> for String {
    fn from(crate_name: CrateName) -> Self {
        crate_name.0.into()
    }
}
impl From<&CrateName> for String {
    fn from(crate_name: &CrateName) -> Self {
        crate_name.0.clone().into()
    }
}

/// Creates a new CrateName rejecting any crate name that
/// has a character on the blacklist.
/// The difference between RA and this implementation is that
/// characters on the blacklist are never allowed; there is no normalization.
impl FromStr for CrateName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let is_invalid = name.chars().any(|n| CHARACTER_BLACK_LIST.contains(&n));
        if is_invalid {
            Err(name.into())
        } else {
            Ok(Self(SmolStr::new(name)))
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateData {
    pub root_file_id: FileId,
    pub name: Option<CrateName>,
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
        self.name.clone().into()
    }
}

impl CrateGraph {
    pub fn get_crate(&self, crate_id: CrateId) -> Option<&CrateData> {
        self.arena.get(&crate_id)
    }

    pub fn add_crate_root(&mut self, file_id: FileId, package_name: Option<CrateName>) -> CrateId {
        let mut roots_with_file_id =
            self.arena.iter().filter(|(_, crate_data)| crate_data.root_file_id == file_id);

        let next_file_id = roots_with_file_id.next();
        if let Some(file_id) = next_file_id {
            return *file_id.0;
        }

        let data =
            CrateData { name: package_name, root_file_id: file_id, dependencies: Vec::new() };
        let crate_id = CrateId::Crate(self.arena.len());
        let prev = self.arena.insert(crate_id, data);
        assert!(prev.is_none());
        crate_id
    }

    pub fn add_stdlib(&mut self, file_id: FileId) -> CrateId {
        let mut roots_with_file_id =
            self.arena.iter().filter(|(_, crate_data)| crate_data.root_file_id == file_id);

        let next_file_id = roots_with_file_id.next();
        if let Some(file_id) = next_file_id {
            return *file_id.0;
        }

        let data = CrateData {
            name: Some(CrateName::from_str("std").unwrap()),
            root_file_id: file_id,
            dependencies: Vec::new(),
        };
        let crate_id = CrateId::Stdlib(self.arena.len());
        let prev = self.arena.insert(crate_id, data);
        assert!(prev.is_none());
        crate_id
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
                go(graph, visited, res, dep.crate_id);
            }
            res.push(source);
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
        self.dependencies.push(Dependency { crate_id, name });
    }
}
impl std::ops::Index<CrateId> for CrateGraph {
    type Output = CrateData;
    fn index(&self, crate_id: CrateId) -> &CrateData {
        &self.arena[&crate_id]
    }
}

/// XXX: This is bare-bone for two reasons:
// There are no display names currently
// The error would be better if it showed the full cyclic dependency, including transitives.
#[allow(dead_code)]
#[derive(Debug)]
pub struct CyclicDependenciesError {
    from: CrateId,
    to: CrateId,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CrateGraph, FileId};

    fn dummy_file_ids(n: usize) -> Vec<FileId> {
        use fm::{FileMap, FILE_EXTENSION};
        let mut fm = FileMap::default();

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
        let crate1 = graph.add_crate_root(file_ids[0], None);
        let crate2 = graph.add_crate_root(file_ids[1], None);
        let crate3 = graph.add_crate_root(file_ids[2], None);

        assert!(graph.add_dep(crate1, "crate2".parse().unwrap(), crate2).is_ok());
        assert!(graph.add_dep(crate2, "crate3".parse().unwrap(), crate3).is_ok());
        assert!(graph.add_dep(crate3, "crate1".parse().unwrap(), crate1).is_err());
    }

    #[test]
    fn it_works() {
        let file_ids = dummy_file_ids(3);
        let file_id_0 = file_ids[0];
        let file_id_1 = file_ids[1];
        let file_id_2 = file_ids[2];
        let mut graph = CrateGraph::default();
        let crate1 = graph.add_crate_root(file_id_0, None);
        let crate2 = graph.add_crate_root(file_id_1, None);
        let crate3 = graph.add_crate_root(file_id_2, None);
        assert!(graph.add_dep(crate1, "crate2".parse().unwrap(), crate2).is_ok());
        assert!(graph.add_dep(crate2, "crate3".parse().unwrap(), crate3).is_ok());
    }
    #[test]
    fn it_works2() {
        let file_ids = dummy_file_ids(3);
        let file_id_0 = file_ids[0];
        let file_id_1 = file_ids[1];
        let file_id_2 = file_ids[2];
        let mut graph = CrateGraph::default();
        let _crate1 = graph.add_crate_root(file_id_0, None);
        let _crate2 = graph.add_crate_root(file_id_1, None);

        // Adding the same file, so the crate should be the same.
        let crate3 = graph.add_crate_root(file_id_2, None);
        let crate3_2 = graph.add_crate_root(file_id_2, None);
        assert_eq!(crate3, crate3_2);
    }
}
