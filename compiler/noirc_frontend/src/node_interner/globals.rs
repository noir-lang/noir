use fm::FileId;
use noirc_errors::Location;

use crate::{
    ast::{Ident, ItemVisibility},
    graph::CrateId,
    hir::{comptime, def_map::LocalModuleId},
    hir_def::stmt::{HirLetStatement, HirStatement},
    node_interner::{DefinitionId, DefinitionInfo, DefinitionKind, Node, StmtId},
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
}

#[derive(Debug, Clone)]
pub enum GlobalValue {
    Unresolved,
    Resolving,
    Resolved(comptime::Value),
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

    pub fn get_global_mut(&mut self, global_id: GlobalId) -> &mut GlobalInfo {
        &mut self.globals[global_id.0]
    }

    pub fn get_global_definition(&self, global_id: GlobalId) -> &DefinitionInfo {
        let global = self.get_global(global_id);
        self.definition(global.definition_id)
    }

    pub fn get_global_definition_mut(&mut self, global_id: GlobalId) -> &mut DefinitionInfo {
        let global = self.get_global(global_id);
        self.definition_mut(global.definition_id)
    }

    pub fn get_all_globals(&self) -> &[GlobalInfo] {
        &self.globals
    }

    /// Try to get the `HirLetStatement` which defines a given global value
    pub fn get_global_let_statement(&self, global: GlobalId) -> Option<HirLetStatement> {
        let global = self.get_global(global);
        let def = self.nodes.get(global.let_statement.0)?;

        match def {
            Node::Statement(hir_stmt) => match hir_stmt {
                HirStatement::Let(let_stmt) => Some(let_stmt.clone()),
                HirStatement::Error => None,
                other => {
                    panic!(
                        "ice: all globals should correspond to a let statement in the interner: {other:?}"
                    )
                }
            },
            _ => panic!("ice: all globals should correspond to a statement in the interner"),
        }
    }
}
