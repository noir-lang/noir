// This has been taken and modified from the rust-analyzer codebase
// For the most part, everything is the same, the differences are quite subtle
// but still present. Moreover, since RA is uses incremental compilation, the usage of this component may differ.
// This version is also simpler due to not having macro_defs or proc_macros
// XXX: Edition may be reintroduced or some sort of versioning

use std::{fmt::Display, str::FromStr};

use fm::FileId;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CrateId {
    Root(usize),
    Crate(usize),
    Stdlib(usize),
    /// The special case of running the compiler against the stdlib.
    /// In that case there's only one crate, and it's both the root
    /// crate and the stdlib crate.
    RootAndStdlib(usize),
    Dummy,
}

impl CrateId {
    pub fn dummy_id() -> CrateId {
        CrateId::Dummy
    }

    pub fn is_stdlib(&self) -> bool {
        match self {
            CrateId::Stdlib(_) | CrateId::RootAndStdlib(_) => true,
            CrateId::Root(_) | CrateId::Crate(_) | CrateId::Dummy => false,
        }
    }

    pub fn is_root(&self) -> bool {
        match self {
            CrateId::Root(_) | CrateId::RootAndStdlib(_) => true,
            CrateId::Stdlib(_) | CrateId::Crate(_) | CrateId::Dummy => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CrateName(SmolStr);

impl CrateName {
    fn is_valid_name(name: &str) -> bool {
        !name.is_empty() && name.chars().all(|n| !CHARACTER_BLACK_LIST.contains(&n))
    }
}

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
        if Self::is_valid_name(name) {
            Ok(Self(SmolStr::new(name)))
        } else {
            Err("Package names must be non-empty and cannot contain hyphens".into())
        }
    }
}

#[cfg(test)]
mod crate_name {
    use super::{CrateName, CHARACTER_BLACK_LIST};

    #[test]
    fn it_rejects_empty_string() {
        assert!(!CrateName::is_valid_name(""));
    }

    #[test]
    fn it_rejects_blacklisted_chars() {
        for bad_char in CHARACTER_BLACK_LIST {
            let bad_char_string = bad_char.to_string();
            assert!(!CrateName::is_valid_name(&bad_char_string));
        }
    }

    #[test]
    fn it_rejects_bad_crate_names_when_deserializing() {
        assert!(serde_json::from_str::<CrateName>("bad-name").is_err());
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrateGraph {
    arena: FxHashMap<CrateId, CrateData>,
}

impl CrateGraph {
    /// Tries to find the requested crate in the current one's dependencies,
    /// otherwise walks down the crate dependency graph from crate_id until we reach it.
    /// This is needed in case a library (lib1) re-export a structure defined in another library (lib2)
    /// In that case, we will get [lib1,lib2] when looking for a struct defined in lib2,
    /// re-exported by lib1 and used by the main crate.
    /// Returns the path from crate_id to target_crate_id
    pub(crate) fn find_dependencies(
        &self,
        crate_id: &CrateId,
        target_crate_id: &CrateId,
    ) -> Option<Vec<String>> {
        self[crate_id]
            .dependencies
            .iter()
            .find_map(|dep| {
                if &dep.crate_id == target_crate_id {
                    Some(vec![dep.name.to_string()])
                } else {
                    None
                }
            })
            .or_else(|| {
                self[crate_id].dependencies.iter().find_map(|dep| {
                    if let Some(mut path) = self.find_dependencies(&dep.crate_id, target_crate_id) {
                        path.insert(0, dep.name.to_string());
                        Some(path)
                    } else {
                        None
                    }
                })
            })
    }
}

/// List of characters that are not allowed in a crate name
/// For example, Hyphen(-) is disallowed as it is similar to underscore(_)
/// and we do not want names that differ by a hyphen
pub const CHARACTER_BLACK_LIST: [char; 1] = ['-'];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateData {
    pub root_file_id: FileId,
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
    pub fn root_crate_id(&self) -> &CrateId {
        self.arena
            .keys()
            .find(|crate_id| crate_id.is_root())
            .expect("ICE: A root crate should exist in the CrateGraph")
    }

    pub fn stdlib_crate_id(&self) -> &CrateId {
        self.arena
            .keys()
            .find(|crate_id| crate_id.is_stdlib())
            .expect("ICE: The stdlib should exist in the CrateGraph")
    }

    pub fn add_crate_root(&mut self, file_id: FileId) -> CrateId {
        for (crate_id, crate_data) in self.arena.iter() {
            if crate_id.is_root() {
                panic!("ICE: Cannot add two crate roots to a graph - use `add_crate` instead");
            }

            if crate_data.root_file_id == file_id {
                panic!("ICE: This FileId was already added to the CrateGraph")
            }
        }

        let data = CrateData { root_file_id: file_id, dependencies: Vec::new() };
        let crate_id = CrateId::Root(self.arena.len());
        let prev = self.arena.insert(crate_id, data);
        assert!(prev.is_none());
        crate_id
    }

    pub fn add_crate(&mut self, file_id: FileId) -> CrateId {
        let mut crates_with_file_id = self
            .arena
            .iter()
            .filter(|(_, crate_data)| crate_data.root_file_id == file_id)
            .peekable();

        let matching_id = crates_with_file_id.next();
        if crates_with_file_id.peek().is_some() {
            panic!("ICE: Found multiple crates with the same FileId");
        }

        match matching_id {
            Some((crate_id @ CrateId::Crate(_), _)) => *crate_id,
            Some((CrateId::Root(_), _)) => {
                panic!("ICE: Tried to re-add the root crate as a regular crate")
            }
            Some((CrateId::Stdlib(_), _)) | Some((CrateId::RootAndStdlib(_), _)) => {
                panic!("ICE: Tried to re-add the stdlib crate as a regular crate")
            }
            Some((CrateId::Dummy, _)) => {
                panic!("ICE: A dummy CrateId should not exist in the CrateGraph")
            }
            None => {
                let data = CrateData { root_file_id: file_id, dependencies: Vec::new() };
                let crate_id = CrateId::Crate(self.arena.len());
                let prev = self.arena.insert(crate_id, data);
                assert!(prev.is_none());
                crate_id
            }
        }
    }

    pub fn add_stdlib(&mut self, file_id: FileId) -> CrateId {
        for (crate_id, crate_data) in self.arena.iter() {
            if crate_id.is_stdlib() {
                panic!("ICE: Cannot add two stdlib crates to a graph - use `add_crate` instead");
            }

            if crate_data.root_file_id == file_id {
                panic!("ICE: This FileId was already added to the CrateGraph")
            }
        }

        let data = CrateData { root_file_id: file_id, dependencies: Vec::new() };
        let crate_id = CrateId::Stdlib(self.arena.len());
        let prev = self.arena.insert(crate_id, data);
        assert!(prev.is_none());
        crate_id
    }

    pub fn add_crate_root_and_stdlib(&mut self, file_id: FileId) -> CrateId {
        for (crate_id, crate_data) in self.arena.iter() {
            if crate_id.is_root() {
                panic!("ICE: Cannot add two crate roots to a graph - use `add_crate` instead");
            }

            if crate_id.is_stdlib() {
                panic!("ICE: Cannot add two stdlib crates to a graph - use `add_crate` instead");
            }

            if crate_data.root_file_id == file_id {
                panic!("ICE: This FileId was already added to the CrateGraph")
            }
        }

        let data = CrateData { root_file_id: file_id, dependencies: Vec::new() };
        let crate_id = CrateId::RootAndStdlib(self.arena.len());
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
impl std::ops::Index<&CrateId> for CrateGraph {
    type Output = CrateData;
    fn index(&self, crate_id: &CrateId) -> &CrateData {
        &self.arena[crate_id]
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
            pth.push(format!("{i}"));
            pth.set_extension(FILE_EXTENSION);
            vec_ids.push(fm.add_file(pth.into(), String::new()));
        }

        vec_ids
    }

    #[test]
    fn detect_cyclic_dependency_indirect() {
        let file_ids = dummy_file_ids(3);

        let mut graph = CrateGraph::default();
        let crate1 = graph.add_crate_root(file_ids[0]);
        let crate2 = graph.add_crate(file_ids[1]);
        let crate3 = graph.add_crate(file_ids[2]);

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
        let crate1 = graph.add_crate_root(file_id_0);
        let crate2 = graph.add_crate(file_id_1);
        let crate3 = graph.add_crate(file_id_2);
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
        let _crate1 = graph.add_crate_root(file_id_0);
        let _crate2 = graph.add_crate(file_id_1);

        // Adding the same file, so the crate should be the same.
        let crate3 = graph.add_crate(file_id_2);
        let crate3_2 = graph.add_crate(file_id_2);
        assert_eq!(crate3, crate3_2);
    }

    #[test]
    #[should_panic = "ICE: Cannot add two crate roots to a graph - use `add_crate` instead"]
    fn panics_if_adding_two_roots() {
        let file_ids = dummy_file_ids(2);
        let mut graph = CrateGraph::default();
        let _ = graph.add_crate_root(file_ids[0]);
        let _ = graph.add_crate_root(file_ids[1]);
    }

    #[test]
    #[should_panic = "ICE: This FileId was already added to the CrateGraph"]
    fn panics_if_adding_existing_file_as_root() {
        let file_ids = dummy_file_ids(1);
        let mut graph = CrateGraph::default();
        let file_id_0 = file_ids[0];
        let _ = graph.add_crate(file_id_0);
        let _ = graph.add_crate_root(file_id_0);
    }

    #[test]
    #[should_panic = "ICE: Cannot add two stdlib crates to a graph - use `add_crate` instead"]
    fn panics_if_adding_two_stdlib() {
        let file_ids = dummy_file_ids(2);
        let mut graph = CrateGraph::default();
        let _ = graph.add_stdlib(file_ids[0]);
        let _ = graph.add_stdlib(file_ids[1]);
    }

    #[test]
    #[should_panic = "ICE: This FileId was already added to the CrateGraph"]
    fn panics_if_adding_existing_file_as_stdlib() {
        let file_ids = dummy_file_ids(1);
        let mut graph = CrateGraph::default();
        let file_id_0 = file_ids[0];
        let _ = graph.add_crate(file_id_0);
        let _ = graph.add_stdlib(file_id_0);
    }

    #[test]
    #[should_panic = "ICE: Tried to re-add the root crate as a regular crate"]
    fn panics_if_adding_root_as_regular() {
        let file_ids = dummy_file_ids(1);
        let mut graph = CrateGraph::default();
        let file_id_0 = file_ids[0];
        let _ = graph.add_crate_root(file_id_0);
        let _ = graph.add_crate(file_id_0);
    }
    #[test]
    #[should_panic = "ICE: Tried to re-add the stdlib crate as a regular crate"]
    fn panics_if_adding_stdlib_as_regular() {
        let file_ids = dummy_file_ids(1);
        let mut graph = CrateGraph::default();
        let file_id_0 = file_ids[0];
        let _ = graph.add_stdlib(file_id_0);
        let _ = graph.add_crate(file_id_0);
    }
}
