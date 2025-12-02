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

use crate::{
    Type,
    ast::Pattern,
    hir::{def_collector::dc_crate::UnresolvedGlobal, resolution::errors::ResolverError},
    hir_def::stmt::HirStatement,
    node_interner::{DependencyId, GlobalId, GlobalValue},
    token::SecondaryAttributeKind,
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Order the set of unresolved globals by their [GlobalId].
    /// This set will be used to determine the ordering in which globals are elaborated.
    pub(super) fn set_unresolved_globals_ordering(&mut self, globals: Vec<UnresolvedGlobal>) {
        for global in globals {
            self.unresolved_globals.insert(global.global_id, global);
        }
    }

    /// Elaborate any globals which were not brought into scope by other items through [Self::elaborate_global_if_unresolved].
    pub(super) fn elaborate_remaining_globals(&mut self) {
        // Start at the first global IDs to maintain the dependency order
        while let Some((_, global)) = self.unresolved_globals.pop_first() {
            self.elaborate_global(global);
        }
    }

    /// Elaborates a global constant definition, performing name resolution, type checking, and compile-time evaluation.
    ///
    /// See the [module-level documentation][self] for more details.
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
        let type_location = let_stmt.r#type.as_ref().map(|typ| typ.location).unwrap_or(location);

        let (global_id, has_errors) = self.with_error_guard(|this| {
            // ABI attributes are only meaningful within contracts, so error if used elsewhere.
            if !this.in_contract() {
                for attr in &let_stmt.attributes {
                    if matches!(attr.kind, SecondaryAttributeKind::Abi(_)) {
                        this.push_err(ResolverError::AbiAttributeOutsideContract {
                            location: attr.location,
                        });
                    }
                }
            }

            // Non-comptime globals must be immutable. Comptime globals can be mutable during
            // compile-time execution, but all globals are immutable at runtime.
            if !let_stmt.comptime && matches!(let_stmt.pattern, Pattern::Mutable(..)) {
                this.push_err(ResolverError::MutableGlobal { location });
            }

            // Elaborate the let statement in a comptime context. This ensures that the expression
            // is type-checked and converted to HIR.
            let (let_statement, _typ) = this.elaborate_in_comptime_context(|this| {
                this.elaborate_let(let_stmt, Some(global_id))
            });

            // References cannot be stored in globals because they would outlive their referents.
            // All data in globals must be owned.
            if let_statement.r#type.contains_reference() {
                this.push_err(ResolverError::ReferencesNotAllowedInGlobals { location });
            }

            if !let_statement.comptime && matches!(let_statement.r#type, Type::Quoted(_)) {
                let typ = let_statement.r#type.to_string();
                let location = type_location;
                this.push_err(ResolverError::ComptimeTypeInNonComptimeGlobal { typ, location });
            }

            let let_statement = HirStatement::Let(let_statement);

            // Replace the placeholder statement that was created during def collection with
            // the fully elaborated HIR statement.
            let statement_id = this.interner.get_global(global_id).let_statement;
            this.interner.replace_statement(statement_id, let_statement);

            global_id
        });

        // Evaluate the global at compile time to get its constant value.
        self.elaborate_comptime_global(global_id, has_errors);

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
    fn elaborate_comptime_global(&mut self, global_id: GlobalId, has_errors: bool) {
        if has_errors {
            self.push_err(crate::hir::comptime::InterpreterError::SkippedDueToEarlierErrors);
            return;
        }

        // Retrieve the HIR let statement that was generated in elaborate_global.
        let let_statement = self
            .interner
            .get_global_let_statement(global_id)
            .expect("Let statement of global should be set by elaborate_global_let");

        let global = self.interner.get_global(global_id);

        let definition_id = global.definition_id;
        let location = global.location;

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

            self.debug_comptime(location, |interner| value.display(interner).to_string());

            // Store the resolved value so it can be used later
            self.interner.get_global_mut(global_id).value = GlobalValue::Resolved(value);
        }
    }

    /// If the given global is unresolved, elaborate it and return true.
    ///
    /// This is used for dependency resolution. When we encounter a reference to a global
    /// while elaborating another item, we check if that global needs to be elaborated first.
    /// Returns true if the global was unresolved and has now been elaborated, false if it was
    /// already elaborated (or doesn't exist in the unresolved set).
    pub(crate) fn elaborate_global_if_unresolved(&mut self, global_id: &GlobalId) -> bool {
        if let Some(global) = self.unresolved_globals.remove(global_id) {
            self.elaborate_global(global);
            true
        } else {
            false
        }
    }
}
