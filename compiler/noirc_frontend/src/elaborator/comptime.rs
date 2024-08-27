use std::{collections::BTreeMap, fmt::Display};

use chumsky::Parser;
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span};

use crate::{
    hir::{
        comptime::{Interpreter, InterpreterError, Value},
        def_collector::{
            dc_crate::{
                CollectedItems, CompilationError, UnresolvedFunctions, UnresolvedStruct,
                UnresolvedTrait, UnresolvedTraitImpl,
            },
            dc_mod,
        },
        resolution::errors::ResolverError,
    },
    hir_def::expr::HirIdent,
    lexer::Lexer,
    macros_api::{
        Expression, ExpressionKind, HirExpression, NodeInterner, SecondaryAttribute, StructId,
    },
    node_interner::{DefinitionKind, DependencyId, FuncId, TraitId},
    parser::{self, TopLevelStatement},
    Type, TypeBindings,
};

use super::{Elaborator, FunctionContext, ResolverMeta};

impl<'context> Elaborator<'context> {
    /// Elaborate an expression from the middle of a comptime scope.
    /// When this happens we require additional information to know
    /// what variables should be in scope.
    pub fn elaborate_item_from_comptime<'a, T>(
        &'a mut self,
        current_function: Option<FuncId>,
        f: impl FnOnce(&mut Elaborator<'a>) -> T,
    ) -> T {
        // Create a fresh elaborator to ensure no state is changed from
        // this elaborator
        let mut elaborator = Elaborator::new(
            self.interner,
            self.def_maps,
            self.crate_id,
            self.debug_comptime_in_file,
            self.enable_arithmetic_generics,
        );

        elaborator.function_context.push(FunctionContext::default());
        elaborator.scopes.start_function();

        if let Some(function) = current_function {
            let meta = elaborator.interner.function_meta(&function);
            elaborator.current_item = Some(DependencyId::Function(function));
            elaborator.crate_id = meta.source_crate;
            elaborator.local_module = meta.source_module;
            elaborator.file = meta.source_file;
            elaborator.introduce_generics_into_scope(meta.all_generics.clone());
        }

        elaborator.populate_scope_from_comptime_scopes();

        let result = f(&mut elaborator);
        elaborator.check_and_pop_function_context();

        self.errors.append(&mut elaborator.errors);
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

    pub(super) fn run_comptime_attributes_on_item(
        &mut self,
        attributes: &[SecondaryAttribute],
        item: Value,
        span: Span,
        generated_items: &mut CollectedItems,
    ) {
        for attribute in attributes {
            if let SecondaryAttribute::Custom(name) = attribute {
                if let Err(error) =
                    self.run_comptime_attribute_on_item(name, item.clone(), span, generated_items)
                {
                    self.errors.push(error);
                }
            }
        }
    }

    fn run_comptime_attribute_on_item(
        &mut self,
        attribute: &str,
        item: Value,
        span: Span,
        generated_items: &mut CollectedItems,
    ) -> Result<(), (CompilationError, FileId)> {
        let location = Location::new(span, self.file);
        let Some((function, arguments)) = Self::parse_attribute(attribute, self.file)? else {
            // Do not issue an error if the attribute is unknown
            return Ok(());
        };

        // Elaborate the function, rolling back any errors generated in case it is unknown
        let error_count = self.errors.len();
        let function = self.elaborate_expression(function).0;
        self.errors.truncate(error_count);

        let definition_id = match self.interner.expression(&function) {
            HirExpression::Ident(ident, _) => ident.id,
            _ => return Ok(()),
        };

        let Some(definition) = self.interner.try_definition(definition_id) else {
            // If there's no such function, don't return an error.
            // This preserves backwards compatibility in allowing custom attributes that
            // do not refer to comptime functions.
            return Ok(());
        };

        let DefinitionKind::Function(function) = definition.kind else {
            return Err((ResolverError::NonFunctionInAnnotation { span }.into(), self.file));
        };

        let mut interpreter = self.setup_interpreter();
        let mut arguments =
            Self::handle_attribute_arguments(&mut interpreter, function, arguments, location)
                .map_err(|error| {
                    let file = error.get_location().file;
                    (error.into(), file)
                })?;

        arguments.insert(0, (item, location));

        let value = interpreter
            .call_function(function, arguments, TypeBindings::new(), location)
            .map_err(|error| error.into_compilation_error_pair())?;

        if value != Value::Unit {
            let items = value
                .into_top_level_items(location, self.interner)
                .map_err(|error| error.into_compilation_error_pair())?;

            self.add_items(items, generated_items, location);
        }

        Ok(())
    }

    /// Parses an attribute in the form of a function call (e.g. `#[foo(a b, c d)]`) into
    /// the function and quoted arguments called (e.g. `("foo", vec![(a b, location), (c d, location)])`)
    #[allow(clippy::type_complexity)]
    fn parse_attribute(
        annotation: &str,
        file: FileId,
    ) -> Result<Option<(Expression, Vec<Expression>)>, (CompilationError, FileId)> {
        let (tokens, mut lexing_errors) = Lexer::lex(annotation);
        if !lexing_errors.is_empty() {
            return Err((lexing_errors.swap_remove(0).into(), file));
        }

        let expression = parser::expression()
            .parse(tokens)
            .map_err(|mut errors| (errors.swap_remove(0).into(), file))?;

        Ok(match expression.kind {
            ExpressionKind::Call(call) => Some((*call.func, call.arguments)),
            ExpressionKind::Variable(_) => Some((expression, Vec::new())),
            _ => None,
        })
    }

    fn handle_attribute_arguments(
        interpreter: &mut Interpreter,
        function: FuncId,
        arguments: Vec<Expression>,
        location: Location,
    ) -> Result<Vec<(Value, Location)>, InterpreterError> {
        let meta = interpreter.elaborator.interner.function_meta(&function);
        let mut parameters = vecmap(&meta.parameters.0, |(_, typ, _)| typ.clone());

        // Remove the initial parameter for the comptime item since that is not included
        // in `arguments` at this point.
        parameters.remove(0);

        // If the function is varargs, push the type of the last slice element N times
        // to account for N extra arguments.
        let modifiers = interpreter.elaborator.interner.function_modifiers(&function);
        let is_varargs = modifiers.attributes.is_varargs();
        let varargs_type = if is_varargs { parameters.pop() } else { None };

        let varargs_elem_type = varargs_type.as_ref().and_then(|t| t.slice_element_type());

        let mut new_arguments = Vec::with_capacity(arguments.len());
        let mut varargs = im::Vector::new();

        for (i, arg) in arguments.into_iter().enumerate() {
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
                    ExpressionKind::Variable(path) => interpreter
                        .elaborator
                        .resolve_trait_by_path(path)
                        .ok_or(InterpreterError::FailedToResolveTraitDefinition { location }),
                    _ => Err(InterpreterError::TraitDefinitionMustBeAPath { location }),
                }?;
                push_arg(Value::TraitDefinition(trait_id));
            } else {
                let expr_id = interpreter.elaborator.elaborate_expression(arg).0;
                push_arg(interpreter.evaluate(expr_id)?);
            }
        }

        if is_varargs {
            let typ = varargs_type.unwrap_or(Type::Error);
            new_arguments.push((Value::Slice(varargs, typ), location));
        }

        Ok(new_arguments)
    }

    fn add_items(
        &mut self,
        items: Vec<TopLevelStatement>,
        generated_items: &mut CollectedItems,
        location: Location,
    ) {
        for item in items {
            self.add_item(item, generated_items, location);
        }
    }

    fn add_item(
        &mut self,
        item: TopLevelStatement,
        generated_items: &mut CollectedItems,
        location: Location,
    ) {
        match item {
            TopLevelStatement::Function(function) => {
                let id = self.interner.push_empty_fn();
                let module = self.module_id();
                self.interner.push_function(id, &function.def, module, location);

                if self.interner.is_in_lsp_mode()
                    && !function.def.is_test()
                    && !function.def.is_private()
                {
                    self.interner.register_function(id, &function.def);
                }

                let functions = vec![(self.local_module, id, function)];
                generated_items.functions.push(UnresolvedFunctions {
                    file_id: self.file,
                    functions,
                    trait_id: None,
                    self_type: None,
                });
            }
            TopLevelStatement::TraitImpl(mut trait_impl) => {
                let (methods, associated_types, associated_constants) =
                    dc_mod::collect_trait_impl_items(
                        self.interner,
                        &mut trait_impl,
                        self.crate_id,
                        self.file,
                        self.local_module,
                    );

                generated_items.trait_impls.push(UnresolvedTraitImpl {
                    file_id: self.file,
                    module_id: self.local_module,
                    trait_generics: trait_impl.trait_generics,
                    trait_path: trait_impl.trait_name,
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
                });
            }
            TopLevelStatement::Global(global) => {
                let (global, error) = dc_mod::collect_global(
                    self.interner,
                    self.def_maps.get_mut(&self.crate_id).unwrap(),
                    global,
                    self.file,
                    self.local_module,
                    self.crate_id,
                );

                generated_items.globals.push(global);
                if let Some(error) = error {
                    self.errors.push(error);
                }
            }
            // Assume that an error has already been issued
            TopLevelStatement::Error => (),

            TopLevelStatement::Module(_)
            | TopLevelStatement::Import(_)
            | TopLevelStatement::Struct(_)
            | TopLevelStatement::Trait(_)
            | TopLevelStatement::Impl(_)
            | TopLevelStatement::TypeAlias(_)
            | TopLevelStatement::SubModule(_) => {
                let item = item.to_string();
                let error = InterpreterError::UnsupportedTopLevelItemUnquote { item, location };
                self.errors.push(error.into_compilation_error_pair());
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
        if Some(location.file) == self.debug_comptime_in_file {
            let displayed_expr = expr_f(self.interner);
            self.errors.push((
                InterpreterError::debug_evaluate_comptime(displayed_expr, location).into(),
                location.file,
            ));
        }
    }

    /// Run all the attributes on each item. The ordering is unspecified to users but currently
    /// we run trait attributes first to (e.g.) register derive handlers before derive is
    /// called on structs.
    /// Returns any new items generated by attributes.
    pub(super) fn run_attributes(
        &mut self,
        traits: &BTreeMap<TraitId, UnresolvedTrait>,
        types: &BTreeMap<StructId, UnresolvedStruct>,
        functions: &[UnresolvedFunctions],
    ) -> CollectedItems {
        let mut generated_items = CollectedItems::default();

        for (trait_id, trait_) in traits {
            let attributes = &trait_.trait_def.attributes;
            let item = Value::TraitDefinition(*trait_id);
            let span = trait_.trait_def.span;
            self.local_module = trait_.module_id;
            self.file = trait_.file_id;
            self.run_comptime_attributes_on_item(attributes, item, span, &mut generated_items);
        }

        for (struct_id, struct_def) in types {
            let attributes = &struct_def.struct_def.attributes;
            let item = Value::StructDefinition(*struct_id);
            let span = struct_def.struct_def.span;
            self.local_module = struct_def.module_id;
            self.file = struct_def.file_id;
            self.run_comptime_attributes_on_item(attributes, item, span, &mut generated_items);
        }

        self.run_attributes_on_functions(functions, &mut generated_items);
        generated_items
    }

    fn run_attributes_on_functions(
        &mut self,
        function_sets: &[UnresolvedFunctions],
        generated_items: &mut CollectedItems,
    ) {
        for function_set in function_sets {
            self.file = function_set.file_id;
            self.self_type = function_set.self_type.clone();

            for (local_module, function_id, function) in &function_set.functions {
                self.local_module = *local_module;
                let attributes = function.secondary_attributes();
                let item = Value::FunctionDefinition(*function_id);
                let span = function.span();
                self.run_comptime_attributes_on_item(attributes, item, span, generated_items);
            }
        }
    }
}
