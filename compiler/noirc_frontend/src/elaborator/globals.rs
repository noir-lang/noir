//! Global constant definition elaboration and comptime evaluation.
//!
//! ## Design
//!
//! Global constants in Noir are elaborated in a two-phase process:
//!
//! ### Name resolution and type Checking and HIR Generation
//! [Elaborator::elaborate_global] validates the global definition and generates its HIR
//! representation. Key constraints enforced:
//! - Globals must be immutable (unless marked `comptime` for compile-time mutation)
//! - Global types cannot contain references
//! - ABI attributes are only valid within contracts
//!
//! ### Comptime Evaluation
//! The [Elaborator::elaborate_comptime_global] function evaluates the global's initializer expression
//! at compile time using the interpreter. The resulting value is stored in the interner and can be used
//! later for compile-time operations such as a type-level arithmetic.
//!
//! ### Dependency Ordering
//! Globals are assumed to be elaborated in dependency order. This means if global `A` references global `B`, then `B`
//! must be elaborated first. It is assumed that the caller of this module has enforced elaborating globals in their dependency order.

use std::collections::HashSet;

use acvm::FieldElement;

use crate::{
    Type,
    ast::Pattern,
    hir::{
        comptime::{Integer, Value},
        def_collector::dc_crate::UnresolvedGlobal,
        resolution::errors::ResolverError,
    },
    hir_def::{expr::HirExpression, stmt::HirStatement},
    node_interner::{DependencyId, GlobalId, GlobalValue},
    token::SecondaryAttributeKind,
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Order the set of unresolved globals by their [GlobalId].
    /// This set will be used to determine the ordering in which globals are elaborated.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn set_unresolved_globals_ordering(&mut self, globals: Vec<UnresolvedGlobal>) {
        for global in globals {
            self.unresolved_globals.insert(global.global_id, global);
        }
    }

    /// Elaborate any globals which were not brought into scope by other items through [Self::elaborate_global_if_unresolved].
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn elaborate_remaining_globals(&mut self) {
        // Start at the first global IDs to maintain the dependency order
        while let Some((_, global)) = self.unresolved_globals.pop_first() {
            self.elaborate_global(global);
        }
    }

    /// Drains every remaining unresolved global, except those listed in `skip`.
    /// Preserves the BTreeMap-ordered drain so that inter-global dependency order is maintained.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_unresolved_globals_skipping(&mut self, skip: &HashSet<GlobalId>) {
        let to_resolve: Vec<GlobalId> =
            self.unresolved_globals.keys().copied().filter(|id| !skip.contains(id)).collect();
        for global_id in to_resolve {
            self.elaborate_global_if_unresolved(&global_id);
        }
    }

    /// Elaborates a global constant definition, performing name resolution, type checking, and compile-time evaluation.
    ///
    /// See the [module-level documentation][self] for more details.
    #[tracing::instrument(level = "trace", skip_all)]
    fn elaborate_global(&mut self, global: UnresolvedGlobal) {
        // Set up the elaboration context for this global. We need to ensure that name resolution
        // happens in the module where the global was defined, not where it's being referenced.
        let old_module = self.local_module.replace(global.module_id);
        let old_item = self.current_item.take();

        let global_id = global.global_id;
        self.current_item = Some(DependencyId::Global(global_id));
        let let_stmt = global.stmt_def;

        // In LSP mode, we need to register the global's name for IDE features like
        // hover information and go-to-definition.
        let name = if self.interner.is_in_lsp_mode() {
            Some(let_stmt.pattern.name_ident().to_string())
        } else {
            None
        };

        let location = let_stmt.pattern.location();

        // ABI attributes are only meaningful within contracts, so error if used elsewhere.
        if !self.in_contract() {
            for attr in &let_stmt.attributes {
                if matches!(attr.kind, SecondaryAttributeKind::Abi(_)) {
                    self.push_err(ResolverError::AbiAttributeOutsideContract {
                        location: attr.location,
                    });
                }
            }
        }

        // Non-comptime globals must be immutable. Comptime globals can be mutable during
        // compile-time execution, but all globals are immutable at runtime.
        if !let_stmt.comptime && matches!(let_stmt.pattern, Pattern::Mutable(..)) {
            self.push_err(ResolverError::MutableGlobal { location });
        }

        self.reset_lvalue_index_counter();
        let (let_statement, _typ) = self.elaborate_let(let_stmt, Some(global_id));

        // References cannot be stored in globals because they would outlive their referents.
        // All data in globals must be owned.
        if let_statement.r#type.contains_reference() {
            self.push_err(ResolverError::ReferencesNotAllowedInGlobals { location });
        }

        let let_statement = HirStatement::Let(let_statement);

        // Replace the placeholder statement that was created during def collection with
        // the fully elaborated HIR statement.
        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        // Evaluate the global at compile time to get its constant value.
        self.elaborate_comptime_global(global_id);

        // Register this global in the LSP database for IDE features.
        if let Some(name) = name {
            self.interner.register_global(global_id, name, location, global.visibility);
        }

        // Restore the previous elaboration context.
        self.local_module = old_module;
        self.current_item = old_item;
    }

    /// Evaluates the global's initializer expression at compile time and stores the resulting value.
    /// The comptime [interpreter][crate::hir::comptime::Interpreter] is used for evaluating the expression.
    ///
    /// See the [module-level documentation][self] for more details.
    #[tracing::instrument(level = "trace", skip_all)]
    fn elaborate_comptime_global(&mut self, global_id: GlobalId) {
        // Retrieve the HIR let statement that was generated in elaborate_global.
        let let_statement = self
            .interner
            .get_global_let_statement(global_id)
            .expect("Let statement of global should be set by elaborate_global_let");

        let global = self.interner.get_global(global_id);

        let definition_id = global.definition_id;
        let location = global.location;
        let name = global.ident.to_string();

        // A CLI `--define NAME=VALUE` override replaces the global's initializer
        // with a value parsed from the command line. Look it up by name; cloning
        // the value releases the borrow on `self.options` before we mutate the
        // interner below.
        let override_value = self
            .options
            .global_overrides
            .iter()
            .find(|(overridden_name, _)| *overridden_name == name)
            .map(|(_, value)| value.clone());

        if let Some(raw) = override_value {
            match global_override_value(&let_statement.r#type, &raw) {
                Ok(value) => {
                    self.debug_comptime(location, |interner, file_manager| {
                        value.display(interner, file_manager).to_string()
                    });
                    self.interner.get_global_mut(global_id).value = GlobalValue::Resolved(value);
                }
                Err(message) => {
                    self.push_err(ResolverError::InvalidGlobalOverride { name, message, location });
                }
            }
            return;
        }

        let expr = self.interner.expression(&let_statement.expression);
        if !matches!(expr, HirExpression::Error) {
            // Globals must be elaborated at the global scope
            let saved_scopes: Vec<_> = self.interner.comptime_scopes.drain(1..).collect();

            let mut interpreter = self.setup_interpreter();

            // Evaluate the global's initializer expression at compile time using the interpreter.
            if let Err(error) = interpreter.evaluate_let(let_statement) {
                self.push_err(error);
            } else {
                // The interpreter has now computed the constant value. Look it up and store it
                // in the interner for use during compilation.
                let value = interpreter
                    .lookup_id(definition_id, location)
                    .expect("The global should be defined since evaluate_let did not error");

                self.debug_comptime(location, |interner, file_manager| {
                    value.display(interner, file_manager).to_string()
                });

                // Store the resolved value so it can be used later
                self.interner.get_global_mut(global_id).value = GlobalValue::Resolved(value);
            }

            self.interner.comptime_scopes.extend(saved_scopes);
        }
    }

    /// If the given global is unresolved, elaborate it and return true.
    ///
    /// This is used for dependency resolution. When we encounter a reference to a global
    /// while elaborating another item, we check if that global needs to be elaborated first.
    /// Returns true if the global was unresolved and has now been elaborated, false if it was
    /// already elaborated (or doesn't exist in the unresolved set).
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn elaborate_global_if_unresolved(&mut self, global_id: &GlobalId) -> bool {
        if let Some(global) = self.unresolved_globals.remove(global_id) {
            self.elaborate_global(global);
            true
        } else {
            false
        }
    }
}

/// Parses a `--define`/`-D` global override string into a comptime [`Value`] of
/// the global's declared type. Only `bool`, `Field`, and integer-typed globals
/// are supported; any other type, an out-of-range value, or a malformed value
/// produces an error message describing the problem.
fn global_override_value(typ: &Type, raw: &str) -> Result<Value, String> {
    let typ = typ.follow_bindings();
    match &typ {
        Type::Bool => match raw.trim() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            other => {
                Err(format!("expected `true` or `false` for a `bool` global, found `{other}`"))
            }
        },
        Type::FieldElement | Type::Integer(..) => {
            let field = parse_override_field(raw)?;
            Integer::try_from_type(field, &typ)
                .map(Value::Integer)
                .ok_or_else(|| format!("value `{}` does not fit in type `{typ}`", raw.trim()))
        }
        other => Err(format!(
            "`--define` overrides are only supported for `bool`, `Field`, and integer globals, \
             but this global has type `{other}`"
        )),
    }
}

/// Parses a (possibly negative) decimal integer override into a [`FieldElement`].
/// Negative values are encoded as negated fields, matching how the comptime
/// interpreter represents signed integers.
fn parse_override_field(raw: &str) -> Result<FieldElement, String> {
    let raw = raw.trim();
    let parse_magnitude =
        |s: &str| s.parse::<u128>().map_err(|_| format!("`{raw}` is not a valid integer"));
    match raw.strip_prefix('-') {
        Some(magnitude) => Ok(-FieldElement::from(parse_magnitude(magnitude)?)),
        None => Ok(FieldElement::from(parse_magnitude(raw)?)),
    }
}
