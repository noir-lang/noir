use std::{collections::BTreeMap, fmt::Display};

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    Type, TypeBindings, UnificationError,
    ast::{Documented, Expression, ExpressionKind},
    hir::{
        comptime::{Interpreter, InterpreterError, Value},
        def_collector::{
            dc_crate::{
                CollectedItems, CompilationError, ModuleAttribute, UnresolvedFunctions,
                UnresolvedStruct, UnresolvedTrait, UnresolvedTraitImpl,
            },
            dc_mod,
        },
        def_map::{LocalModuleId, ModuleId},
        resolution::errors::ResolverError,
    },
    hir_def::expr::{HirExpression, HirIdent},
    node_interner::{DefinitionKind, DependencyId, FuncId, NodeInterner, TraitId, TypeId},
    parser::{Item, ItemKind},
    token::{MetaAttribute, MetaAttributeName, SecondaryAttribute, SecondaryAttributeKind},
};

use super::{ElaborateReason, Elaborator, FunctionContext, ResolverMeta};

#[derive(Debug, Copy, Clone)]
struct AttributeContext {
    // The module where generated items should be added
    module: LocalModuleId,
    // The module where the attribute is located
    attribute_module: LocalModuleId,
}

type CollectedAttributes = Vec<(FuncId, Value, Vec<Expression>, AttributeContext, Location)>;

impl AttributeContext {
    fn new(module: LocalModuleId) -> Self {
        Self { module, attribute_module: module }
    }
}

impl<'context> Elaborator<'context> {
    /// Elaborate an expression from the middle of a comptime scope.
    /// When this happens we require additional information to know
    /// what variables should be in scope.
    pub fn elaborate_item_from_comptime_in_function<'a, T>(
        &'a mut self,
        current_function: Option<FuncId>,
        reason: Option<ElaborateReason>,
        f: impl FnOnce(&mut Elaborator<'a>) -> T,
    ) -> T {
        self.elaborate_item_from_comptime(reason, f, |elaborator| {
            if let Some(function) = current_function {
                let meta = elaborator.interner.function_meta(&function);
                elaborator.current_item = Some(DependencyId::Function(function));
                elaborator.crate_id = meta.source_crate;
                elaborator.local_module = meta.source_module;
                elaborator.introduce_generics_into_scope(meta.all_generics.clone());
            }
        })
    }

    pub fn elaborate_item_from_comptime_in_module<'a, T>(
        &'a mut self,
        module: ModuleId,
        reason: Option<ElaborateReason>,
        f: impl FnOnce(&mut Elaborator<'a>) -> T,
    ) -> T {
        self.elaborate_item_from_comptime(reason, f, |elaborator| {
            elaborator.current_item = None;
            elaborator.crate_id = module.krate;
            elaborator.local_module = module.local_id;
        })
    }

    fn elaborate_item_from_comptime<'a, T>(
        &'a mut self,
        reason: Option<ElaborateReason>,
        f: impl FnOnce(&mut Elaborator<'a>) -> T,
        setup: impl FnOnce(&mut Elaborator<'a>),
    ) -> T {
        // Create a fresh elaborator to ensure no state is changed from
        // this elaborator
        let mut elaborator = Elaborator::new(
            self.interner,
            self.def_maps,
            self.usage_tracker,
            self.crate_graph,
            self.interpreter_output,
            self.crate_id,
            self.interpreter_call_stack.clone(),
            self.options,
            self.elaborate_reasons.clone(),
        );

        elaborator.function_context.push(FunctionContext::default());
        elaborator.scopes.start_function();

        elaborator.local_module = self.local_module;

        setup(&mut elaborator);

        elaborator.populate_scope_from_comptime_scopes();

        let result = f(&mut elaborator);
        elaborator.check_and_pop_function_context();

        let mut errors = std::mem::take(&mut elaborator.errors);
        if let Some(reason) = reason {
            errors = vecmap(errors, |error| {
                CompilationError::ComptimeError(reason.to_macro_error(error))
            });
        };

        self.errors.extend(errors);
        result
    }

    fn populate_scope_from_comptime_scopes(&mut self) {
        // Take the comptime scope to be our runtime scope.
        // Iterate from global scope to the most local scope so that the
        // later definitions will naturally shadow the former.
        for scope in &self.interner.comptime_scopes {
            for definition_id in scope.keys() {
                let definition = self.interner.definition(*definition_id);
                let name = definition.name.clone();
                let location = definition.location;

                let scope = self.scopes.get_mut_scope();
                let ident = HirIdent::non_trait_method(*definition_id, location);
                let meta = ResolverMeta { ident, num_times_used: 0, warn_if_unused: false };
                scope.add_key_value(name.clone(), meta);
            }
        }
    }

    fn collect_comptime_attributes_on_item(
        &mut self,
        attributes: &[SecondaryAttribute],
        item: Value,
        attribute_context: AttributeContext,
        attributes_to_run: &mut CollectedAttributes,
    ) {
        for attribute in attributes {
            self.collect_comptime_attribute_on_item(
                attribute,
                &item,
                attribute_context,
                attributes_to_run,
            );
        }
    }

    fn collect_comptime_attribute_on_item(
        &mut self,
        attribute: &SecondaryAttribute,
        item: &Value,
        attribute_context: AttributeContext,
        attributes_to_run: &mut CollectedAttributes,
    ) {
        if let SecondaryAttributeKind::Meta(meta) = &attribute.kind {
            self.elaborate_in_comptime_context(|this| {
                if let Err(error) = this.collect_comptime_attribute_name_on_item(
                    meta,
                    attribute.location,
                    item.clone(),
                    attribute_context,
                    attributes_to_run,
                ) {
                    this.push_err(error);
                }
            });
        }
    }

    /// Resolve an attribute to the function it refers to and add it to `attributes_to_run`
    fn collect_comptime_attribute_name_on_item(
        &mut self,
        attribute: &MetaAttribute,
        location: Location,
        item: Value,
        attribute_context: AttributeContext,
        attributes_to_run: &mut CollectedAttributes,
    ) -> Result<(), CompilationError> {
        self.local_module = attribute_context.attribute_module;

        let kind = match &attribute.name {
            MetaAttributeName::Path(path) => ExpressionKind::Variable(path.clone()),
            MetaAttributeName::Resolved(expr_id) => ExpressionKind::Resolved(*expr_id),
        };

        let function = Expression { kind, location };
        let arguments = attribute.arguments.clone();

        // Elaborate the function, rolling back any errors generated in case it is unknown
        let error_count = self.errors.len();
        let function_string = function.to_string();
        let function = self.elaborate_expression(function).0;
        self.errors.truncate(error_count);

        let definition_id = match self.interner.expression(&function) {
            HirExpression::Ident(ident, _) => ident.id,
            _ => {
                let error = ResolverError::AttributeFunctionIsNotAPath {
                    function: function_string,
                    location,
                };
                return Err(error.into());
            }
        };

        let Some(definition) = self.interner.try_definition(definition_id) else {
            let error =
                ResolverError::AttributeFunctionNotInScope { name: function_string, location };
            return Err(error.into());
        };

        let DefinitionKind::Function(function) = definition.kind else {
            return Err(ResolverError::NonFunctionInAnnotation { location }.into());
        };

        attributes_to_run.push((function, item, arguments, attribute_context, location));
        Ok(())
    }

    fn run_attribute(
        &mut self,
        attribute_context: AttributeContext,
        function: FuncId,
        arguments: Vec<Expression>,
        item: Value,
        location: Location,
        generated_items: &mut CollectedItems,
    ) -> Result<(), CompilationError> {
        self.local_module = attribute_context.module;

        let mut interpreter = self.setup_interpreter();
        let mut arguments = Self::handle_attribute_arguments(
            &mut interpreter,
            &item,
            function,
            arguments,
            location,
        )
        .map_err(CompilationError::from)?;

        arguments.insert(0, (item, location));

        let value = interpreter
            .call_function(function, arguments, TypeBindings::default(), location)
            .map_err(CompilationError::from)?;

        self.debug_comptime(location, |interner| value.display(interner).to_string());

        if value != Value::Unit {
            let items =
                value.into_top_level_items(location, self).map_err(CompilationError::from)?;

            self.add_items(items, generated_items, location);
        }

        Ok(())
    }

    fn handle_attribute_arguments(
        interpreter: &mut Interpreter,
        item: &Value,
        function: FuncId,
        arguments: Vec<Expression>,
        location: Location,
    ) -> Result<Vec<(Value, Location)>, InterpreterError> {
        let meta = interpreter.elaborator.interner.function_meta(&function);

        let mut parameters = vecmap(&meta.parameters.0, |(_, typ, _)| typ.clone());

        if parameters.is_empty() {
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: 0,
                actual: arguments.len() + 1,
                location,
            });
        }

        let expected_type = item.get_type();
        let expected_type = expected_type.as_ref();

        if &parameters[0] != expected_type {
            return Err(InterpreterError::TypeMismatch {
                expected: parameters[0].clone(),
                actual: expected_type.clone(),
                location,
            });
        }

        // Remove the initial parameter for the comptime item since that is not included
        // in `arguments` at this point.
        parameters.remove(0);

        // If the function is varargs, push the type of the last slice element N times
        // to account for N extra arguments.
        let modifiers = interpreter.elaborator.interner.function_modifiers(&function);
        let is_varargs = modifiers.attributes.has_varargs();
        let varargs_type = if is_varargs { parameters.pop() } else { None };

        let varargs_elem_type = varargs_type.as_ref().and_then(|t| t.slice_element_type());

        let mut new_arguments = Vec::with_capacity(arguments.len());
        let mut varargs = im::Vector::new();

        for (i, arg) in arguments.into_iter().enumerate() {
            let arg_location = arg.location;
            let param_type = parameters.get(i).or(varargs_elem_type).unwrap_or(&Type::Error);

            let mut push_arg = |arg| {
                if i >= parameters.len() {
                    varargs.push_back(arg);
                } else {
                    new_arguments.push((arg, location));
                }
            };

            if *param_type == Type::Quoted(crate::QuotedType::TraitDefinition) {
                let trait_id = match arg.kind {
                    ExpressionKind::Variable(path) => {
                        let path = interpreter.elaborator.validate_path(path);
                        interpreter
                            .elaborator
                            .resolve_trait_by_path(path)
                            .ok_or(InterpreterError::FailedToResolveTraitDefinition { location })
                    }
                    _ => Err(InterpreterError::TraitDefinitionMustBeAPath { location }),
                }?;
                push_arg(Value::TraitDefinition(trait_id));
            } else {
                let (expr_id, expr_type) = interpreter.elaborator.elaborate_expression(arg);
                if let Err(UnificationError) = expr_type.unify(param_type) {
                    return Err(InterpreterError::TypeMismatch {
                        expected: param_type.clone(),
                        actual: expr_type,
                        location: arg_location,
                    });
                }
                push_arg(interpreter.evaluate(expr_id)?);
            };
        }

        if is_varargs {
            let typ = varargs_type.unwrap_or(Type::Error);
            new_arguments.push((Value::Slice(varargs, typ), location));
        }

        Ok(new_arguments)
    }

    fn add_items(
        &mut self,
        items: Vec<Item>,
        generated_items: &mut CollectedItems,
        location: Location,
    ) {
        self.with_elaborate_reason(ElaborateReason::RunningAttribute(location), |elaborator| {
            for item in items {
                elaborator.add_item(item, generated_items, location);
            }
        });
    }

    pub(crate) fn add_item(
        &mut self,
        item: Item,
        generated_items: &mut CollectedItems,
        location: Location,
    ) {
        match item.kind {
            ItemKind::Function(function) => {
                let module_id = self.module_id();

                if let Some(id) = dc_mod::collect_function(
                    self.interner,
                    self.def_maps.get_mut(&self.crate_id).unwrap(),
                    self.usage_tracker,
                    &function,
                    module_id,
                    item.doc_comments,
                    &mut self.errors,
                ) {
                    let functions = vec![(self.local_module, id, function)];
                    generated_items.functions.push(UnresolvedFunctions {
                        file_id: location.file,
                        functions,
                        trait_id: None,
                        self_type: None,
                    });
                }
            }
            ItemKind::TraitImpl(mut trait_impl) => {
                let (methods, associated_types, associated_constants) =
                    dc_mod::collect_trait_impl_items(
                        self.interner,
                        &mut trait_impl,
                        self.crate_id,
                        location.file,
                        self.local_module,
                    );

                generated_items.trait_impls.push(UnresolvedTraitImpl {
                    file_id: location.file,
                    module_id: self.local_module,
                    r#trait: trait_impl.r#trait,
                    object_type: trait_impl.object_type,
                    methods,
                    generics: trait_impl.impl_generics,
                    where_clause: trait_impl.where_clause,
                    associated_types,
                    associated_constants,

                    // These last fields are filled in later
                    trait_id: None,
                    impl_id: None,
                    resolved_object_type: None,
                    resolved_generics: Vec::new(),
                    resolved_trait_generics: Vec::new(),
                    unresolved_associated_types: Vec::new(),
                });
            }
            ItemKind::Global(global, visibility) => {
                let (global, error) = dc_mod::collect_global(
                    self.interner,
                    self.def_maps.get_mut(&self.crate_id).unwrap(),
                    self.usage_tracker,
                    Documented::new(global, item.doc_comments),
                    visibility,
                    location.file,
                    self.local_module,
                    self.crate_id,
                );

                generated_items.globals.push(global);
                if let Some(error) = error {
                    self.push_err(error);
                }
            }
            ItemKind::Struct(struct_def) => {
                if let Some((type_id, the_struct)) = dc_mod::collect_struct(
                    self.interner,
                    self.def_maps.get_mut(&self.crate_id).unwrap(),
                    self.usage_tracker,
                    Documented::new(struct_def, item.doc_comments),
                    self.local_module,
                    self.crate_id,
                    &mut self.errors,
                ) {
                    generated_items.structs.insert(type_id, the_struct);
                }
            }
            ItemKind::Enum(enum_def) => {
                if let Some((type_id, the_enum)) = dc_mod::collect_enum(
                    self.interner,
                    self.def_maps.get_mut(&self.crate_id).unwrap(),
                    self.usage_tracker,
                    Documented::new(enum_def, item.doc_comments),
                    location.file,
                    self.local_module,
                    self.crate_id,
                    &mut self.errors,
                ) {
                    generated_items.enums.insert(type_id, the_enum);
                }
            }
            ItemKind::Impl(r#impl) => {
                let module = self.module_id();
                dc_mod::collect_impl(
                    self.interner,
                    generated_items,
                    r#impl,
                    location.file,
                    module,
                    &mut self.errors,
                );
            }

            ItemKind::ModuleDecl(_)
            | ItemKind::Import(..)
            | ItemKind::Trait(_)
            | ItemKind::TypeAlias(_)
            | ItemKind::Submodules(_)
            | ItemKind::InnerAttribute(_) => {
                let location = item.location;
                let item = item.kind.to_string();
                let error = InterpreterError::UnsupportedTopLevelItemUnquote { item, location };
                self.push_err(error);
            }
        }
    }

    pub fn setup_interpreter<'local>(&'local mut self) -> Interpreter<'local, 'context> {
        let current_function = match self.current_item {
            Some(DependencyId::Function(function)) => Some(function),
            _ => None,
        };
        Interpreter::new(self, self.crate_id, current_function)
    }

    pub(super) fn debug_comptime<T: Display, F: FnMut(&mut NodeInterner) -> T>(
        &mut self,
        location: Location,
        mut expr_f: F,
    ) {
        if Some(location.file) == self.options.debug_comptime_in_file {
            let displayed_expr = expr_f(self.interner);
            let error: CompilationError =
                InterpreterError::debug_evaluate_comptime(displayed_expr, location).into();
            self.push_err(error);
        }
    }

    /// Run all the attributes on each item in the crate in source-order.
    /// Source-order is defined as running all child modules before their parent modules are run.
    /// Child modules of a parent are run in order of their `mod foo;` declarations in the parent.
    pub(super) fn run_attributes(
        &mut self,
        traits: &BTreeMap<TraitId, UnresolvedTrait>,
        types: &BTreeMap<TypeId, UnresolvedStruct>,
        functions: &[UnresolvedFunctions],
        module_attributes: &[ModuleAttribute],
    ) {
        let mut attributes_to_run = Vec::new();

        for (trait_id, trait_) in traits {
            let attributes = &trait_.trait_def.attributes;
            let item = Value::TraitDefinition(*trait_id);
            let context = AttributeContext::new(trait_.module_id);
            self.collect_comptime_attributes_on_item(
                attributes,
                item,
                context,
                &mut attributes_to_run,
            );
        }

        for (struct_id, struct_def) in types {
            let attributes = &struct_def.struct_def.attributes;
            let item = Value::TypeDefinition(*struct_id);
            let context = AttributeContext::new(struct_def.module_id);
            self.collect_comptime_attributes_on_item(
                attributes,
                item,
                context,
                &mut attributes_to_run,
            );
        }

        self.collect_attributes_on_functions(functions, &mut attributes_to_run);
        self.collect_attributes_on_modules(module_attributes, &mut attributes_to_run);

        self.sort_attributes_by_run_order(&mut attributes_to_run);

        // run
        for (attribute, item, args, context, location) in attributes_to_run {
            let mut generated_items = CollectedItems::default();
            self.elaborate_in_comptime_context(|this| {
                if let Err(error) = this.run_attribute(
                    context,
                    attribute,
                    args,
                    item,
                    location,
                    &mut generated_items,
                ) {
                    this.push_err(error);
                }
            });

            if !generated_items.is_empty() {
                let reason = ElaborateReason::RunningAttribute(location);
                self.with_elaborate_reason(reason, |elaborator| {
                    elaborator.elaborate_items(generated_items);
                });
            }
        }
    }

    fn sort_attributes_by_run_order(&self, attributes: &mut CollectedAttributes) {
        let module_order = self.def_maps[&self.crate_id].get_module_topological_order();

        // Sort each attribute by (module, location in file) so that we can execute in
        // the order they were defined in, running attributes in child modules first.
        attributes.sort_by_key(|(_, _, _, ctx, location)| {
            (module_order[&ctx.attribute_module], location.span.start())
        });
    }

    fn collect_attributes_on_modules(
        &mut self,
        module_attributes: &[ModuleAttribute],
        attributes_to_run: &mut CollectedAttributes,
    ) {
        for module_attribute in module_attributes {
            let local_id = module_attribute.module_id;
            let module_id = ModuleId { krate: self.crate_id, local_id };
            let item = Value::ModuleDefinition(module_id);
            let attribute = &module_attribute.attribute;

            let context = AttributeContext {
                module: module_attribute.module_id,
                attribute_module: module_attribute.attribute_module_id,
            };

            self.collect_comptime_attribute_on_item(attribute, &item, context, attributes_to_run);
        }
    }

    fn collect_attributes_on_functions(
        &mut self,
        function_sets: &[UnresolvedFunctions],
        attributes_to_run: &mut CollectedAttributes,
    ) {
        for function_set in function_sets {
            self.self_type = function_set.self_type.clone();

            for (local_module, function_id, function) in &function_set.functions {
                let context = AttributeContext::new(*local_module);
                let attributes = function.secondary_attributes();
                let item = Value::FunctionDefinition(*function_id);
                self.collect_comptime_attributes_on_item(
                    attributes,
                    item,
                    context,
                    attributes_to_run,
                );
            }
        }
    }

    /// Perform the given function in a comptime context.
    /// This will set the `in_comptime_context` flag on `self` as well as
    /// push a new function context to resolve any trait constraints early.
    pub(super) fn elaborate_in_comptime_context<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let old_comptime_value = std::mem::replace(&mut self.in_comptime_context, true);
        // We have to push a new FunctionContext so that we can resolve any constraints
        // in this comptime block early before the function as a whole finishes elaborating.
        // Otherwise the interpreter below may find expressions for which the underlying trait
        // call is not yet solved for.
        self.function_context.push(Default::default());

        let result = f(self);

        self.check_and_pop_function_context();
        self.in_comptime_context = old_comptime_value;
        result
    }

    /// True if we're currently within a `comptime` block, function, or global
    pub(super) fn in_comptime_context(&self) -> bool {
        self.in_comptime_context
            || match self.current_item {
                Some(DependencyId::Function(id)) => {
                    self.interner.function_modifiers(&id).is_comptime
                }
                Some(DependencyId::Global(id)) => self.interner.get_global_definition(id).comptime,
                _ => false,
            }
    }

    pub(crate) fn with_elaborate_reason<F, T>(&mut self, reason: ElaborateReason, f: F) -> T
    where
        F: FnOnce(&mut Elaborator) -> T,
    {
        self.elaborate_reasons.push_back(reason);
        let previous_errors = std::mem::take(&mut self.errors);

        let value = f(self);

        let new_errors = std::mem::take(&mut self.errors);
        let new_errors = self.wrap_errors_in_macro_error(new_errors);
        self.errors = previous_errors;
        self.push_errors(new_errors);
        self.elaborate_reasons.pop_back();

        value
    }

    fn wrap_errors_in_macro_error(&self, errors: Vec<CompilationError>) -> Vec<CompilationError> {
        vecmap(errors, |error| self.wrap_error_in_macro_error(error))
    }

    fn wrap_error_in_macro_error(&self, mut error: CompilationError) -> CompilationError {
        for reason in self.elaborate_reasons.iter().rev() {
            error = CompilationError::ComptimeError(reason.to_macro_error(error));
        }
        error
    }
}
