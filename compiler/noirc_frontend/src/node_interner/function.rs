use noirc_errors::Location;

use crate::{
    Type,
    ast::{FunctionDefinition, ItemVisibility},
    hir::def_map::ModuleId,
    hir_def::{
        expr::{HirExpression, HirIdent},
        function::{FuncMeta, HirFunction},
        stmt::{HirLetStatement, HirStatement},
    },
    node_interner::{
        DefinitionId, DefinitionKind, ExprId, FuncId, FunctionModifiers, Node, ReferenceId, TraitId,
    },
    token::Attributes,
};

use super::NodeInterner;

impl NodeInterner {
    /// Intern an empty function.
    pub fn push_empty_fn(&mut self) -> FuncId {
        self.push_fn(HirFunction::empty())
    }
    /// Updates the underlying interned Function.
    ///
    /// This method is used as we eagerly intern empty functions to
    /// generate function identifiers and then we update at a later point in
    /// time.
    pub fn update_fn(&mut self, func_id: FuncId, hir_func: HirFunction) {
        let def =
            self.nodes.get_mut(func_id.0).expect("ice: all function ids should have definitions");

        let func = match def {
            Node::Function(func) => func,
            _ => panic!("ice: all function ids should correspond to a function in the interner"),
        };
        *func = hir_func;
    }

    pub fn find_function(&self, function_name: &str) -> Option<FuncId> {
        self.func_meta
            .iter()
            .find(|(func_id, _func_meta)| self.function_name(func_id) == function_name)
            .map(|(func_id, _meta)| *func_id)
    }

    ///Interns a function's metadata.
    ///
    /// Note that the FuncId has been created already.
    /// See ModCollector for it's usage.
    pub fn push_fn_meta(&mut self, func_data: FuncMeta, func_id: FuncId) {
        self.func_meta.insert(func_id, func_data);
    }

    pub fn push_function(
        &mut self,
        id: FuncId,
        function: &FunctionDefinition,
        module: ModuleId,
        location: Location,
    ) -> DefinitionId {
        let name_location = Location::new(function.name.span(), location.file);
        let modifiers = FunctionModifiers {
            name: function.name.to_string(),
            visibility: function.visibility,
            attributes: function.attributes.clone(),
            is_unconstrained: function.is_unconstrained,
            generic_count: function.generics.len(),
            is_comptime: function.is_comptime,
            name_location,
        };
        let definition_id = self.push_function_definition(id, modifiers, module, location);
        self.add_definition_location(ReferenceId::Function(id), name_location);
        definition_id
    }

    pub fn push_function_definition(
        &mut self,
        func: FuncId,
        modifiers: FunctionModifiers,
        module: ModuleId,
        location: Location,
    ) -> DefinitionId {
        let name = modifiers.name.clone();
        let comptime = modifiers.is_comptime;
        self.function_modifiers.insert(func, modifiers);
        self.function_modules.insert(func, module);
        self.push_definition(name, false, comptime, DefinitionKind::Function(func), location)
    }

    pub fn set_function_trait(&mut self, func: FuncId, self_type: Type, trait_id: TraitId) {
        self.func_id_to_trait.insert(func, (self_type, trait_id));
    }

    pub fn get_function_trait(&self, func: &FuncId) -> Option<(Type, TraitId)> {
        self.func_id_to_trait.get(func).cloned()
    }

    /// Returns the visibility of the given function.
    ///
    /// The underlying function_visibilities map is populated during def collection,
    /// so this function can be called anytime afterward.
    pub fn function_visibility(&self, func: FuncId) -> ItemVisibility {
        self.function_modifiers[&func].visibility
    }

    /// Returns the module this function was defined within
    pub fn function_module(&self, func: FuncId) -> ModuleId {
        self.function_modules[&func]
    }

    /// Returns the [`FuncId`] corresponding to the function referred to by `expr_id`,
    /// _iff_ the expression is an [HirExpression::Ident] with a `Function` definition,
    /// or if `follow_redirects` is `true`, then an immutable `Local` or `Global`,
    /// ultimately pointing at a `Function`.
    ///
    /// Returns `None` for all other cases (tuples, array, mutable variables, etc.).
    pub(crate) fn lookup_function_from_expr(
        &self,
        expr: &ExprId,
        follow_redirects: bool,
    ) -> Option<FuncId> {
        if let HirExpression::Ident(HirIdent { id, .. }, _) = self.expression(expr) {
            match self.try_definition(id).map(|def| &def.kind) {
                Some(DefinitionKind::Function(func_id)) => Some(*func_id),
                Some(DefinitionKind::Local(Some(expr_id))) if follow_redirects => {
                    self.lookup_function_from_expr(expr_id, follow_redirects)
                }
                Some(DefinitionKind::Global(global_id)) if follow_redirects => {
                    let info = self.get_global(*global_id);
                    let HirStatement::Let(HirLetStatement { expression, .. }) =
                        self.statement(&info.let_statement)
                    else {
                        unreachable!("global refers to a let statement");
                    };
                    self.lookup_function_from_expr(&expression, follow_redirects)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Returns the interned HIR function corresponding to `func_id`
    //
    // Cloning HIR structures is cheap, so we return owned structures
    pub fn function(&self, func_id: &FuncId) -> HirFunction {
        let def = self.nodes.get(func_id.0).expect("ice: all function ids should have definitions");

        match def {
            Node::Function(func) => func.clone(),
            _ => panic!("ice: all function ids should correspond to a function in the interner"),
        }
    }

    /// Returns the interned meta data corresponding to `func_id`
    pub fn function_meta(&self, func_id: &FuncId) -> &FuncMeta {
        self.func_meta.get(func_id).expect("ice: all function ids should have metadata")
    }

    pub fn function_meta_mut(&mut self, func_id: &FuncId) -> &mut FuncMeta {
        self.func_meta.get_mut(func_id).expect("ice: all function ids should have metadata")
    }

    pub fn try_function_meta(&self, func_id: &FuncId) -> Option<&FuncMeta> {
        self.func_meta.get(func_id)
    }

    pub fn function_ident(&self, func_id: &FuncId) -> crate::ast::Ident {
        let name = self.function_name(func_id).to_owned();
        let location = self.function_meta(func_id).name.location;
        crate::ast::Ident::new(name, location)
    }

    pub fn function_name(&self, func_id: &FuncId) -> &str {
        &self.function_modifiers[func_id].name
    }

    pub fn function_modifiers(&self, func_id: &FuncId) -> &FunctionModifiers {
        &self.function_modifiers[func_id]
    }

    pub fn function_modifiers_mut(&mut self, func_id: &FuncId) -> &mut FunctionModifiers {
        self.function_modifiers.get_mut(func_id).expect("func_id should always have modifiers")
    }

    pub fn function_attributes(&self, func_id: &FuncId) -> &Attributes {
        &self.function_modifiers[func_id].attributes
    }
}
