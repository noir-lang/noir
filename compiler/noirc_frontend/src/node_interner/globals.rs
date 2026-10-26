use std::collections::{HashMap, HashSet};

use fm::{FileId, FileMap};
use noirc_errors::Location;

use crate::{
    ast::{Ident, ItemVisibility},
    graph::CrateId,
    hir::{comptime, def_map::LocalModuleId},
    hir_def::stmt::{HirLetStatement, HirStatement},
    node_interner::{DefinitionId, DefinitionInfo, DefinitionKind, ExprId, Node, StmtId},
    token::SecondaryAttribute,
};

use super::NodeInterner;

/// An ID for a global value
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct GlobalId(usize);

#[derive(Debug, Clone)]
pub struct GlobalInfo {
    pub id: GlobalId,
    pub definition_id: DefinitionId,
    pub ident: Ident,
    pub visibility: ItemVisibility,
    pub local_id: LocalModuleId,
    pub crate_id: CrateId,
    pub location: Location,
    pub let_statement: StmtId,
    pub value: GlobalValue,
    /// The resolved value lowered to runtime HIR, or the error lowering it produced. Set by
    /// [`NodeInterner::resolve_global`] for every global that is not `comptime`; `comptime`
    /// globals are only reachable from comptime code, so they keep `None`.
    pub runtime_value: Option<Result<ExprId, comptime::InterpreterError>>,
    /// `true` if this global is the synthetic global a fieldless enum variant
    /// (e.g. `Foo::Spam`) is lowered to, rather than a user-declared global.
    pub is_enum_variant: bool,
}

#[derive(Debug, Clone)]
pub enum GlobalValue {
    Unresolved,
    Resolving,
    Resolved(comptime::Value),
}

/// The value of every evaluated `comptime mut global` at one point of a compilation.
pub(crate) type ComptimeGlobalsSnapshot = Vec<(GlobalId, comptime::Value)>;

/// What [`crate::hir::Context::check_crates`] records about `comptime mut global`s while
/// checking several crates in one context, so that each crate sees the state a compilation of
/// its own would.
#[derive(Debug, Default)]
pub(crate) struct ComptimeStateTracking {
    /// Each crate's own mutable globals right after the crate was elaborated.
    own_globals_after_crate: HashMap<CrateId, ComptimeGlobalsSnapshot>,
    /// Every mutable global right before each crate's own items were elaborated.
    globals_before_items: HashMap<CrateId, ComptimeGlobalsSnapshot>,
    /// Crates whose own elaboration changed a mutable global of another crate.
    stateful_crates: HashSet<CrateId>,
}

impl NodeInterner {
    pub fn next_global_id(&mut self) -> GlobalId {
        GlobalId(self.globals.len())
    }

    #[allow(clippy::too_many_arguments)]
    fn push_global(
        &mut self,
        ident: Ident,
        local_id: LocalModuleId,
        crate_id: CrateId,
        let_statement: StmtId,
        file: FileId,
        attributes: Vec<SecondaryAttribute>,
        mutable: bool,
        comptime: bool,
        visibility: ItemVisibility,
    ) -> GlobalId {
        let id = GlobalId(self.globals.len());
        let location = Location::new(ident.span(), file);
        let name = ident.to_string();

        let definition_id =
            self.push_definition(name, mutable, comptime, DefinitionKind::Global(id), location);

        self.globals.push(GlobalInfo {
            id,
            definition_id,
            ident,
            local_id,
            crate_id,
            let_statement,
            location,
            visibility,
            value: GlobalValue::Unresolved,
            runtime_value: None,
            is_enum_variant: false,
        });
        self.global_attributes.insert(id, attributes);
        id
    }

    /// Intern an empty global. Used for collecting globals before they're defined
    #[allow(clippy::too_many_arguments)]
    pub fn push_empty_global(
        &mut self,
        name: Ident,
        local_id: LocalModuleId,
        crate_id: CrateId,
        file: FileId,
        attributes: Vec<SecondaryAttribute>,
        mutable: bool,
        comptime: bool,
        visibility: ItemVisibility,
    ) -> GlobalId {
        let statement = self.push_stmt_full(HirStatement::Error, name.location());

        self.push_global(
            name, local_id, crate_id, statement, file, attributes, mutable, comptime, visibility,
        )
    }

    pub fn get_global(&self, global_id: GlobalId) -> &GlobalInfo {
        &self.globals[global_id.0]
    }

    /// Returns `true` if `global_id` is the synthetic global a fieldless enum variant
    /// (e.g. `Foo::Spam`) is lowered to, as opposed to a user-declared global.
    pub fn is_enum_variant_global(&self, global_id: GlobalId) -> bool {
        self.get_global(global_id).is_enum_variant
    }

    /// Store `value` as the resolved value of `global_id`.
    ///
    /// Unless the global is `comptime`, the value is also lowered to runtime HIR here, once, for
    /// monomorphization to read. A lowering error is kept rather than reported: only a global used
    /// in runtime code needs a runtime value, and monomorphization reports the error when it
    /// reaches one.
    pub(crate) fn resolve_global(
        &mut self,
        global_id: GlobalId,
        value: comptime::Value,
        comptime: bool,
        files: &FileMap,
    ) {
        let runtime_value = (!comptime).then(|| {
            let location = self.get_global(global_id).location;
            value.clone().into_runtime_hir_expression(self, files, location)
        });
        let global = self.get_global_mut(global_id);
        global.value = GlobalValue::Resolved(value);
        global.runtime_value = runtime_value;
    }

    pub fn get_global_mut(&mut self, global_id: GlobalId) -> &mut GlobalInfo {
        &mut self.globals[global_id.0]
    }

    pub fn get_global_definition(&self, global_id: GlobalId) -> &DefinitionInfo {
        let global = self.get_global(global_id);
        self.definition(global.definition_id)
    }

    pub fn get_all_globals(&self) -> &[GlobalInfo] {
        &self.globals
    }

    /// Deep-copies the current value of every evaluated `comptime mut global`, or only those of
    /// `owner`. The stdlib's are left out: its registries only accumulate the derive handlers of
    /// traits a crate can name, so no crate needs them reset or re-registered.
    fn snapshot_mutable_comptime_globals(&self, owner: Option<CrateId>) -> ComptimeGlobalsSnapshot {
        self.globals
            .iter()
            .filter(|global| !global.crate_id.is_stdlib())
            .filter(|global| owner.is_none_or(|owner| global.crate_id == owner))
            .filter(|global| self.definition(global.definition_id).mutable)
            .filter_map(|global| match &global.value {
                GlobalValue::Resolved(value) => Some((global.id, value.deep_clone())),
                GlobalValue::Unresolved | GlobalValue::Resolving => None,
            })
            .collect()
    }

    /// Whether any global in `before` holds a different value now.
    fn mutable_comptime_globals_changed(&self, before: &ComptimeGlobalsSnapshot) -> bool {
        before.iter().any(|(id, old)| {
            !matches!(&self.globals[id.0].value, GlobalValue::Resolved(current) if current == old)
        })
    }

    /// Starts recording what [`ComptimeStateTracking`] describes, if not recording already.
    pub(crate) fn start_tracking_comptime_state(&mut self) {
        self.comptime_state.get_or_insert_default();
    }

    /// Records the state of mutable comptime globals right before `crate_id`'s own items are
    /// elaborated, that is, once its dependencies are. Does nothing unless tracking.
    pub(crate) fn record_comptime_globals_before_items(&mut self, crate_id: CrateId) {
        if self.comptime_state.is_some() {
            let snapshot = self.snapshot_mutable_comptime_globals(None);
            let tracking = self.comptime_state.as_mut().expect("tracking was checked above");
            tracking.globals_before_items.insert(crate_id, snapshot);
        }
    }

    /// Records `crate_id` as elaborated: its own mutable globals' values, and whether elaborating
    /// it changed another crate's. Does nothing unless tracking.
    pub(crate) fn record_comptime_globals_after_crate(&mut self, crate_id: CrateId) {
        let Some(tracking) = &self.comptime_state else {
            return;
        };
        let stateful = tracking
            .globals_before_items
            .get(&crate_id)
            .is_some_and(|before| self.mutable_comptime_globals_changed(before));
        let own = self.snapshot_mutable_comptime_globals(Some(crate_id));
        let tracking = self.comptime_state.as_mut().expect("tracking was checked above");
        if stateful {
            tracking.stateful_crates.insert(crate_id);
        }
        tracking.own_globals_after_crate.insert(crate_id, own);
    }

    /// Sets every crate's own mutable comptime globals back to their values from right after
    /// that crate was elaborated. Does nothing unless tracking.
    pub(crate) fn restore_own_comptime_globals(&mut self) {
        if let Some(tracking) = &self.comptime_state {
            for snapshot in tracking.own_globals_after_crate.values() {
                restore_mutable_comptime_globals(&mut self.globals, snapshot);
            }
        }
    }

    /// Whether elaborating `crate_id` changed a `comptime mut global` of another crate, such as
    /// a library registering its types with a macro library. Sharing the elaboration of such a
    /// crate between the crates that depend on it would lose those writes for all but the first.
    pub fn is_stateful_crate(&self, crate_id: CrateId) -> bool {
        self.comptime_state
            .as_ref()
            .is_some_and(|tracking| tracking.stateful_crates.contains(&crate_id))
    }

    /// Sets every mutable comptime global back to its value from right before `crate_id`'s own
    /// items were elaborated, so that elaborating the crate again starts from the state a fresh
    /// compilation of it would have reached. Does nothing unless the crate was checked with
    /// tracking.
    pub fn restore_comptime_globals_before_items(&mut self, crate_id: CrateId) {
        let tracking = self.comptime_state.as_ref();
        if let Some(snapshot) =
            tracking.and_then(|tracking| tracking.globals_before_items.get(&crate_id))
        {
            restore_mutable_comptime_globals(&mut self.globals, snapshot);
        }
    }

    /// Whether comptime state is being tracked (see [`ComptimeStateTracking`]).
    #[cfg(test)]
    pub(crate) fn tracks_comptime_state(&self) -> bool {
        self.comptime_state.is_some()
    }

    /// The current value of an evaluated `comptime mut global`, read through the cell a mutable
    /// global's value lives in.
    pub fn mutable_comptime_global_value(&self, global_id: GlobalId) -> Option<comptime::Value> {
        match &self.globals[global_id.0].value {
            GlobalValue::Resolved(comptime::Value::Pointer(cell, _, _)) => {
                Some(cell.borrow().clone())
            }
            GlobalValue::Resolved(value) => Some(value.clone()),
            GlobalValue::Unresolved | GlobalValue::Resolving => None,
        }
    }

    /// Try to get the `HirLetStatement` which defines a given global value
    pub fn get_global_let_statement(&self, global: GlobalId) -> Option<HirLetStatement> {
        let global = self.get_global(global);
        let def = self.nodes.get(global.let_statement.0)?;

        match def {
            Node::Statement(hir_stmt) => match hir_stmt {
                HirStatement::Let(let_stmt) => Some(let_stmt.clone()),
                HirStatement::Error | HirStatement::TraitAssociatedConstant => None,
                _other => None,
            },
            _ => panic!("ice: all globals should correspond to a statement in the interner"),
        }
    }
}

/// Sets every global in `snapshot` back to the value it had when the snapshot was taken.
fn restore_mutable_comptime_globals(
    globals: &mut [GlobalInfo],
    snapshot: &ComptimeGlobalsSnapshot,
) {
    for (id, value) in snapshot {
        globals[id.0].value = GlobalValue::Resolved(value.deep_clone());
    }
}
