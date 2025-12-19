//! The comptime interpreter is a tree-walking interpreter used for evaluating Noir code
//! at compile-time. It is typically triggered (by the elaborator) in one of four scenarios:
//! 1. A `comptime {}` block
//!   - Everything in the block is interpreted
//! 2. A macro call `foo!()`
//!   - The interpreter calls the function `foo` and inlines the resulting `Quoted` code at the callsite.
//! 3. An attribute call `#[my_attr] struct Foo {}`
//!   - The interpreter calls the function `my_attr` and, if `my_attr` returns a `Quoted` value,
//!     inlines the resulting `Quoted` code.
//! 4. A global `global FOO = expr;`
//!   - The interpreter evaluates `expr` to simplify the global to a constant.
//!   - This means any side-effects in `expr` will be performed at compile-time (!).
//!     - This may change in the future.
//!
//! The interpreter operates on the HIR which only requires interpreted code to be elaborated
//! before-hand, it does not need to be translated into another IR. Operating on high-level
//! code like this makes the interpreter more predictable, hopefully limiting bugs, but does
//! make it rather slow in practice.
//!
//! Since unquoting macros may result in new variables in scope, the elaborator must run on that
//! code after the interpreter is run. Yet the requirement that the interpreter runs on HIR means
//! the interpreter must run in the middle of the elaborator. The usual flow is for the elaborator
//! to elaborate as it goes, creating new HIR. Then when it sees a `comptime {}` block or other
//! item that must be interpreted, it elaborates the entire item, creates and runs an [Interpreter]
//! on it, inlines the result, and continues elaborating the rest of the code.
//!
//! Also note that although it runs on code that has already been elaborated, in general it is
//! still possible to invoke the interpreter on code which contains errors, such as type errors.
//! For this reason, the interpreter must still perform error-checking at least in cases where
//! we cannot continue otherwise. This can result in similar errors being issued for the same
//! code. For example, a function's body may fail to type check, but that same function may
//! be called in the interpreter later on where we'd presumably halt with a similar error.
//! [InterpreterError::ArgumentCountMismatch] is an example of such an error.

use std::collections::VecDeque;
use std::{collections::hash_map::Entry, rc::Rc};

use acvm::AcirField;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;
use rustc_hash::FxHashMap as HashMap;

use crate::TypeVariable;
use crate::ast::{BinaryOpKind, FunctionKind, IntegerBitSize, UnaryOp};
use crate::elaborator::{ElaborateReason, Elaborator, ElaboratorOptions};
use crate::hir::Context;
use crate::hir::def_map::ModuleId;
use crate::hir::type_check::TypeCheckError;
use crate::hir_def::expr::TraitItem;
use crate::monomorphization::{
    perform_impl_bindings, perform_instantiation_bindings, resolve_trait_item,
    undo_instantiation_bindings,
};
use crate::node_interner::GlobalValue;
use crate::shared::Signedness;
use crate::signed_field::SignedField;
use crate::token::{FmtStrFragment, Tokens};
use crate::{
    Shared, Type, TypeBindings,
    hir_def::{
        expr::{
            HirArrayLiteral, HirBlockExpression, HirCallExpression, HirCastExpression,
            HirConstrainExpression, HirConstructorExpression, HirEnumConstructorExpression,
            HirExpression, HirIdent, HirIfExpression, HirIndexExpression, HirInfixExpression,
            HirLambda, HirLiteral, HirMemberAccess, HirPrefixExpression, ImplKind,
        },
        function::FunctionBody,
        stmt::{
            HirAssignStatement, HirForStatement, HirLValue, HirLetStatement, HirPattern,
            HirStatement,
        },
        types::Kind,
    },
    node_interner::{DefinitionId, DefinitionKind, ExprId, FuncId, StmtId},
};

use super::errors::{IResult, InterpreterError};
use super::value::{Closure, Value, unwrap_rc};

mod builtin;
mod cast;
mod foreign;
mod infix;
mod unquote;

/// Maximum depth of evaluation, limiting recursion during comptime as well as
/// expression depth. The goal is to be able to provide Noir stack traces if
/// we run out, rather than a Rust backtrace.
///
/// Ideally we would like the recursion limit to be 1000, to match what we do in ACIR,
/// however due to the overhead of the interpreter itself, which recursively evaluates
/// expressions, this needs to be lower.
///
/// Furthermore, since every expression is evaluated via recursion in the interpreter,
/// a deeply nested expression can also hit the Rust stack limit. We would like to
/// provide a Noir stack trace for these as well.
///
/// If, in the future, we refactor the interpreter to be iterative, rather than use recursion,
/// we could have a separate limit just for comptime call stack depth.
const MAX_EVALUATION_DEPTH: usize = 300;

#[allow(unused)]
pub struct Interpreter<'local, 'interner> {
    /// To expand macros the Interpreter needs access to the Elaborator
    pub elaborator: &'local mut Elaborator<'interner>,

    /// True if the interpreter is currently in a loop (in the current function).
    /// Used only to error if break/continue are used outside a loop.
    in_loop: bool,

    /// The current function being interpreted. This may be `None` if we're interpreting
    /// the rhs of a global.
    current_function: Option<FuncId>,

    /// Maps each generic to the binding it has in the current callstack.
    /// Since the interpreter monomorphizes as it interprets, we can bind over the same generic
    /// multiple times. Without the outer Vec, when one of these inner functions exits we would
    /// unbind the generic completely instead of resetting it to its previous binding.
    bound_generics: Vec<HashMap<TypeVariable, (Type, Kind)>>,

    /// Current evaluation depth.
    evaluation_depth: usize,
}

impl<'local, 'interner> Interpreter<'local, 'interner> {
    pub(crate) fn new(
        elaborator: &'local mut Elaborator<'interner>,
        current_function: Option<FuncId>,
    ) -> Self {
        Self {
            elaborator,
            current_function,
            bound_generics: Vec::new(),
            in_loop: false,
            evaluation_depth: 0,
        }
    }

    /// Call the given function with the given arguments and return the result.
    ///
    /// This will handle internal details like binding generics and error handling.
    /// Note that running code which resulted in previous errors during elaboration
    /// may result in similar errors being issued again by the interpreter.
    pub(crate) fn call_function(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        mut instantiation_bindings: TypeBindings,
        location: Location,
    ) -> IResult<Value> {
        let trait_method = self.elaborator.interner.get_trait_item_id(function);

        // To match the monomorphizer, we need to call follow_bindings on each of
        // the instantiation bindings before we unbind the generics from the previous function.
        // This is because the instantiation bindings refer to variables from the call site.
        for (_type_var, kind, binding) in instantiation_bindings.values_mut() {
            *kind = kind.follow_bindings();
            *binding = binding.follow_bindings();
        }

        self.unbind_generics_from_previous_function();
        perform_instantiation_bindings(&instantiation_bindings);

        let impl_bindings =
            perform_impl_bindings(self.elaborator.interner, trait_method, function, location)?;

        self.remember_bindings(&instantiation_bindings, &impl_bindings);
        self.elaborator.push_interpreter_call_stack(location)?;

        let result = self.call_function_inner(function, arguments, location);

        self.elaborator.pop_interpreter_call_stack();
        undo_instantiation_bindings(impl_bindings);
        undo_instantiation_bindings(instantiation_bindings);
        self.rebind_generics_from_previous_function();
        result
    }

    /// Helper to check parameter count and dispatch on the function kind to run the function.
    fn call_function_inner(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        location: Location,
    ) -> IResult<Value> {
        let meta = self.elaborator.interner.function_meta(&function);
        if meta.parameters.len() != arguments.len() {
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: meta.parameters.len(),
                actual: arguments.len(),
                location,
            });
        }

        if meta.kind != FunctionKind::Normal {
            let return_type = meta.return_type().follow_bindings();
            return self.call_special(function, arguments, return_type, location);
        }

        // Don't change the current function scope if we're in a #[use_callers_scope] function.
        // This will affect where `Expression::resolve`, `Quoted::as_type`, and similar functions resolve.
        let old_function = self.current_function;
        let modifiers = self.elaborator.interner.function_modifiers(&function);
        if !modifiers.attributes.has_use_callers_scope() {
            self.current_function = Some(function);
        }

        let result = self.call_user_defined_function(function, arguments, location);
        self.current_function = old_function;
        result
    }

    /// Call a non-builtin function
    fn call_user_defined_function(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        location: Location,
    ) -> IResult<Value> {
        let meta = self.elaborator.interner.function_meta(&function);
        let parameters = meta.parameters.0.clone();
        let previous_state = self.enter_function();

        for ((parameter, typ, _), (argument, arg_location)) in parameters.iter().zip(arguments) {
            let result = self.define_pattern(parameter, typ, argument, arg_location);
            if let Err(err) = result {
                self.exit_function(previous_state);
                return Err(err);
            }
        }

        let function_body = match self.get_function_body(function, location) {
            Ok(body) => body,
            Err(err) => {
                self.exit_function(previous_state);
                return Err(err);
            }
        };
        let result = self.evaluate(function_body);
        self.exit_function(previous_state);
        result
    }

    /// Try to retrieve a function's body.
    /// If the function has not yet been resolved this will attempt to lazily resolve it.
    /// Afterwards, if the function's body is still not known or the function is still
    /// in a Resolving state we issue an error.
    fn get_function_body(&mut self, function: FuncId, location: Location) -> IResult<ExprId> {
        let meta = self.elaborator.interner.function_meta(&function);
        match self.elaborator.interner.function(&function).try_as_expr() {
            Some(body) => Ok(body),
            None => {
                if matches!(&meta.function_body, FunctionBody::Unresolved(..)) {
                    self.elaborate_in_function(None, None, |elaborator| {
                        elaborator.elaborate_function(function);
                    });

                    // Recursive call - this will now hit the Some(body) branch
                    self.get_function_body(function, location)
                } else {
                    let function = self.elaborator.interner.function_name(&function).to_owned();
                    Err(InterpreterError::ComptimeDependencyCycle { function, location })
                }
            }
        }
    }

    /// Helper to elaborate the given item in the given function's context. If None is passed,
    /// the global context is used. This function will temporarily unbind any generics from the
    /// previous function call if they exist.
    fn elaborate_in_function<T>(
        &mut self,
        function: Option<FuncId>,
        reason: Option<ElaborateReason>,
        f: impl FnOnce(&mut Elaborator) -> T,
    ) -> T {
        // Why do we only unbind generics from the previous function here?
        self.unbind_generics_from_previous_function();
        let result = self.elaborator.elaborate_item_from_comptime_in_function(function, reason, f);
        self.rebind_generics_from_previous_function();
        result
    }

    /// Run the given function with an elaborator in the context of the given module.
    /// Temporarily undoes any generics from the previous function.
    fn elaborate_in_module<T>(
        &mut self,
        module: ModuleId,
        reason: Option<ElaborateReason>,
        f: impl FnOnce(&mut Elaborator) -> T,
    ) -> T {
        self.unbind_generics_from_previous_function();
        let result = self.elaborator.elaborate_item_from_comptime_in_module(module, reason, f);
        self.rebind_generics_from_previous_function();
        result
    }

    /// Calls a builtin, foreign, or oracle function (not all oracles are supported).
    ///
    /// This will ignore any oracles starting with "__debug"
    fn call_special(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        return_type: Type,
        location: Location,
    ) -> IResult<Value> {
        let attributes = self.elaborator.interner.function_attributes(&function);
        let func_attrs = &attributes.function()
            .expect("all builtin functions must contain a function attribute which contains the opcode which it links to").kind;

        if let Some(builtin) = func_attrs.builtin() {
            self.call_builtin(builtin.clone().as_str(), arguments, return_type, location)
        } else if let Some(foreign) = func_attrs.foreign() {
            self.call_foreign(foreign.clone().as_str(), arguments, return_type, location)
        } else if let Some(oracle) = func_attrs.oracle() {
            if oracle == "print" {
                self.print_oracle(arguments)
            // Ignore debugger functions
            } else if oracle.starts_with("__debug") {
                Ok(Value::Unit)
            } else {
                let item = format!("Comptime evaluation for oracle functions like '{oracle}'");
                Err(InterpreterError::Unimplemented { item, location })
            }
        } else {
            let name = self.elaborator.interner.function_name(&function);
            unreachable!("Non-builtin, low-level or oracle builtin fn '{name}'")
        }
    }

    /// Call a closure value with the given arguments and environment, returning the result.
    fn call_closure(
        &mut self,
        lambda: HirLambda,
        environment: Vec<Value>,
        arguments: Vec<(Value, Location)>,
        function_scope: Option<FuncId>,
        module_scope: ModuleId,
        call_location: Location,
    ) -> IResult<Value> {
        // Set the closure's scope to that of the function it was originally evaluated in
        let old_module = self.elaborator.replace_module(module_scope);
        let old_function = std::mem::replace(&mut self.current_function, function_scope);

        self.elaborator.push_interpreter_call_stack(call_location)?;

        let result = self.call_closure_inner(lambda, environment, arguments, call_location);

        self.elaborator.pop_interpreter_call_stack();

        self.current_function = old_function;
        let Some(old_module) = old_module else {
            // The module should always be set by the time we're interpreting comptime code
            panic!("ICE: Expected local_module to be set when calling a closure");
        };
        self.elaborator.replace_module(old_module);
        result
    }

    /// Performs the bulk of the work for calling a closure function.
    /// This function is very similar to [Self::call_user_defined_function]
    /// with the main difference being handling of `closure.captures`.
    fn call_closure_inner(
        &mut self,
        closure: HirLambda,
        environment: Vec<Value>,
        arguments: Vec<(Value, Location)>,
        call_location: Location,
    ) -> IResult<Value> {
        let previous_state = self.enter_function();

        if closure.parameters.len() != arguments.len() {
            self.exit_function(previous_state);
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: closure.parameters.len(),
                actual: arguments.len(),
                location: call_location,
            });
        }

        let parameters = closure.parameters.iter().zip(arguments);
        for ((parameter, typ), (argument, arg_location)) in parameters {
            let result = self.define_pattern(parameter, typ, argument, arg_location);
            if let Err(err) = result {
                self.exit_function(previous_state);
                return Err(err);
            }
        }

        for (param, arg) in closure.captures.into_iter().zip(environment) {
            self.define(param.ident.id, arg);
        }

        let result = self.evaluate(closure.body);

        self.exit_function(previous_state);
        result
    }

    /// Enters a function, pushing a new scope and resetting any required state.
    /// Returns the previous values of the internal state, to be reset when
    /// [Self::exit_function] is called.
    pub(super) fn enter_function(&mut self) -> (bool, Vec<HashMap<DefinitionId, Value>>) {
        // Drain every scope except the global scope
        let mut scope = Vec::new();
        if self.elaborator.interner.comptime_scopes.len() > 1 {
            scope = self.elaborator.interner.comptime_scopes.drain(1..).collect();
        }
        self.push_scope();
        (std::mem::take(&mut self.in_loop), scope)
    }

    /// Resets the per-function state to the value previously returned by [Self::enter_function]
    pub(super) fn exit_function(&mut self, mut state: (bool, Vec<HashMap<DefinitionId, Value>>)) {
        self.in_loop = state.0;

        // Keep only the global scope
        self.elaborator.interner.comptime_scopes.truncate(1);
        self.elaborator.interner.comptime_scopes.append(&mut state.1);
    }

    /// Pushes a new scope to define any variables in.
    ///
    /// Note that the first scope is always expected to be the global scope shared by all
    /// crates, which should never be popped.
    pub(super) fn push_scope(&mut self) {
        self.elaborator.interner.comptime_scopes.push(HashMap::default());
    }

    /// Pops the topmost scope.
    ///
    /// Note that the first scope is expected to be the global scope of comptime values
    /// shared between all crates, which should never be popped.
    pub(super) fn pop_scope(&mut self) {
        self.elaborator.interner.comptime_scopes.pop().expect("Expected a scope to exist");
        assert!(!self.elaborator.interner.comptime_scopes.is_empty());
    }

    /// Returns the current scope to define comptime variables in.
    /// The stack of scopes is always non-empty so this should never panic.
    fn current_scope_mut(&mut self) -> &mut HashMap<DefinitionId, Value> {
        // the global scope is always at index zero, so this is always Some
        self.elaborator.interner.comptime_scopes.last_mut().unwrap()
    }

    /// Unbinds all of the generics at the top of `self.bound_generics`, then push
    /// an empty set of bindings to become the new top of the stack.
    fn unbind_generics_from_previous_function(&mut self) {
        if let Some(bindings) = self.bound_generics.last() {
            for (var, (_, kind)) in bindings {
                var.unbind(var.id(), kind.clone());
            }
        }
        // Push a new bindings list for the current function
        self.bound_generics.push(HashMap::default());
    }

    /// Pops the top of `self.bound_generics` then takes the new bindings at the
    /// top of that stack and force-binds each.
    fn rebind_generics_from_previous_function(&mut self) {
        // Remove the currently bound generics first.
        self.bound_generics.pop();

        if let Some(bindings) = self.bound_generics.last() {
            for (var, (binding, _kind)) in bindings {
                var.force_bind(binding.clone());
            }
        }
    }

    /// Adds all of the given `main_bindings` and `impl_bindings` to the top of
    /// `self.bound_generics`. Note that this will not actually perform any of the type bindings.
    fn remember_bindings(&mut self, main_bindings: &TypeBindings, impl_bindings: &TypeBindings) {
        let bound_generics = self
            .bound_generics
            .last_mut()
            .expect("remember_bindings called with no bound_generics on the stack");

        for (var, kind, binding) in main_bindings.values() {
            bound_generics.insert(var.clone(), (binding.follow_bindings(), kind.clone()));
        }

        for (var, kind, binding) in impl_bindings.values() {
            bound_generics.insert(var.clone(), (binding.follow_bindings(), kind.clone()));
        }
    }

    /// Defines a pattern, putting all variables contained within the pattern in the current scope.
    pub(super) fn define_pattern(
        &mut self,
        pattern: &HirPattern,
        typ: &Type,
        argument: Value,
        location: Location,
    ) -> IResult<()> {
        match pattern {
            HirPattern::Identifier(identifier) => {
                self.define(identifier.id, argument);
                Ok(())
            }
            HirPattern::Mutable(pattern, _) => {
                // Create a mutable reference to store to
                let argument = Value::Pointer(Shared::new(argument), true, true);
                self.define_pattern(pattern, typ, argument, location)
            }
            HirPattern::Tuple(pattern_fields, _) => {
                let typ = &typ.follow_bindings();

                match (argument, typ) {
                    (Value::Tuple(fields), Type::Tuple(type_fields))
                        if fields.len() == pattern_fields.len() =>
                    {
                        for ((pattern, typ), argument) in
                            pattern_fields.iter().zip(type_fields).zip(fields)
                        {
                            let argument = argument.borrow().clone();
                            self.define_pattern(pattern, typ, argument, location)?;
                        }
                        Ok(())
                    }
                    (value, _) => {
                        let actual = value.get_type().into_owned();
                        Err(InterpreterError::TypeMismatch {
                            expected: typ.to_string(),
                            actual,
                            location,
                        })
                    }
                }
            }
            HirPattern::Struct(_struct_type, pattern_fields, _) => match argument {
                Value::Struct(fields, struct_type) if fields.len() == pattern_fields.len() => {
                    for (field_name, field_pattern) in pattern_fields {
                        let field = fields.get(field_name.as_string()).ok_or_else(|| {
                            InterpreterError::ExpectedStructToHaveField {
                                typ: struct_type.clone(),
                                field_name: field_name.to_string(),
                                location,
                            }
                        })?;

                        let field = field.borrow();
                        let field_type = field.get_type().into_owned();
                        self.define_pattern(field_pattern, &field_type, field.clone(), location)?;
                    }
                    Ok(())
                }
                value => Err(InterpreterError::TypeMismatch {
                    expected: typ.to_string(),
                    actual: value.get_type().into_owned(),
                    location,
                }),
            },
        }
    }

    /// Define a new variable in the current scope
    fn define(&mut self, id: DefinitionId, argument: Value) {
        self.current_scope_mut().insert(id, argument);
    }

    /// Mutate an existing variable, potentially from a prior scope
    fn mutate(&mut self, id: DefinitionId, argument: Value, location: Location) -> IResult<()> {
        // If the id is a dummy, assume the error was already issued elsewhere
        if id == DefinitionId::dummy_id() {
            return Ok(());
        }

        for scope in self.elaborator.interner.comptime_scopes.iter_mut().rev() {
            if let Entry::Occupied(mut entry) = scope.entry(id) {
                match entry.get() {
                    Value::Pointer(reference, true, _) => {
                        // We can't store to the reference directly, we need to check if the value
                        // is a struct or tuple to store to each field instead. This is so any
                        // references to these fields are also updated.
                        Self::store_flattened(reference.clone(), argument);
                    }
                    _ => {
                        entry.insert(argument);
                    }
                }
                return Ok(());
            }
        }
        Err(InterpreterError::VariableNotInScope { location })
    }

    /// Lookup the comptime value of the given variable
    pub(super) fn lookup(&self, ident: &HirIdent) -> IResult<Value> {
        self.lookup_id(ident.id, ident.location)
    }

    /// Lookup the comptime value of the given definition
    pub fn lookup_id(&self, id: DefinitionId, location: Location) -> IResult<Value> {
        for scope in self.elaborator.interner.comptime_scopes.iter().rev() {
            if let Some(value) = scope.get(&id) {
                return Ok(value.clone());
            }
        }

        if id == DefinitionId::dummy_id() {
            Err(InterpreterError::VariableNotInScope { location })
        } else {
            let name = self.elaborator.interner.definition_name(id).to_string();
            Err(InterpreterError::NonComptimeVarReferenced { name, location })
        }
    }

    /// Evaluate an expression and return the result.
    /// This will automatically dereference a mutable variable if used.
    pub fn evaluate(&mut self, id: ExprId) -> IResult<Value> {
        // If comptime evaluation has been halted, don't execute anything
        if self.elaborator.comptime_evaluation_halted {
            return Err(InterpreterError::SkippedDueToEarlierErrors);
        }

        // Skip expressions that had errors during elaboration and halt all future execution
        if self.elaborator.interner.exprs_with_errors.contains(&id) {
            self.elaborator.comptime_evaluation_halted = true;
            return Err(InterpreterError::SkippedDueToEarlierErrors);
        }

        match self.evaluate_no_dereference(id)? {
            Value::Pointer(elem, true, _) => Ok(elem.unwrap_or_clone().move_struct()),
            other => Ok(other.move_struct()),
        }
    }

    /// Evaluating a mutable variable will dereference it automatically.
    /// This function should be used when that is not desired - e.g. when
    /// compiling a `&mut var` expression to grab the original reference.
    fn evaluate_no_dereference(&mut self, id: ExprId) -> IResult<Value> {
        if self.evaluation_depth >= MAX_EVALUATION_DEPTH {
            let location = self.elaborator.interner.expr_location(&id);
            return Err(InterpreterError::EvaluationDepthOverflow {
                location,
                call_stack: self.elaborator.interpreter_call_stack().clone(),
            });
        }
        self.evaluation_depth += 1;
        let result = match self.elaborator.interner.expression(&id) {
            HirExpression::Ident(ident, _) => self.evaluate_ident(ident, id),
            HirExpression::Literal(literal) => self.evaluate_literal(literal, id),
            HirExpression::Block(block) => self.evaluate_block(block),
            HirExpression::Prefix(prefix) => self.evaluate_prefix(prefix, id),
            HirExpression::Infix(infix) => self.evaluate_infix(infix, id),
            HirExpression::Index(index) => self.evaluate_index(index, id),
            HirExpression::Constructor(constructor) => self.evaluate_constructor(constructor, id),
            HirExpression::MemberAccess(access) => self.evaluate_access(access, id),
            HirExpression::Call(call) => self.evaluate_call(call, id),
            HirExpression::Constrain(constrain) => self.evaluate_constrain(constrain),
            HirExpression::Cast(cast) => self.evaluate_cast(&cast, id),
            HirExpression::If(if_) => self.evaluate_if(if_),
            HirExpression::Match(_) => todo!("Evaluate match in comptime code"),
            HirExpression::Tuple(tuple) => self.evaluate_tuple(tuple),
            HirExpression::Lambda(lambda) => self.evaluate_lambda(lambda, id),
            HirExpression::Quote(tokens) => self.evaluate_quote(tokens),
            HirExpression::Unsafe(block) => self.evaluate_block(block),
            HirExpression::EnumConstructor(constructor) => {
                self.evaluate_enum_constructor(constructor, id)
            }
            HirExpression::Unquote(_) => {
                // An Unquote expression being found is indicative of a macro being
                // expanded within another comptime fn which we don't currently support.
                let location = self.elaborator.interner.expr_location(&id);
                Err(InterpreterError::UnquoteFoundDuringEvaluation { location })
            }
            HirExpression::Error => {
                let location = self.elaborator.interner.expr_location(&id);
                Err(InterpreterError::ErrorNodeEncountered { location })
            }
        };
        self.evaluation_depth -= 1;
        result
    }

    /// Evaluates a variable
    pub(super) fn evaluate_ident(&mut self, ident: HirIdent, id: ExprId) -> IResult<Value> {
        let definition = self.elaborator.interner.try_definition(ident.id).ok_or_else(|| {
            let location = self.elaborator.interner.expr_location(&id);
            InterpreterError::VariableNotInScope { location }
        })?;

        if let ImplKind::TraitItem(item) = ident.impl_kind {
            return self.evaluate_trait_item(item, id);
        }

        match &definition.kind {
            DefinitionKind::Function(function_id) => {
                let typ = self.elaborator.interner.id_type(id).follow_bindings();
                let bindings = self.elaborator.interner.try_get_instantiation_bindings(id);
                let bindings = Rc::new(bindings.map_or(TypeBindings::default(), Clone::clone));
                Ok(Value::Function(*function_id, typ, bindings))
            }
            DefinitionKind::Local(_) => self.lookup(&ident),
            DefinitionKind::Global(global_id) => {
                // Avoid resetting the value if it is already known
                let global_id = *global_id;
                let global_info = self.elaborator.interner.get_global(global_id);
                match &global_info.value {
                    GlobalValue::Resolved(value) => Ok(value.clone()),
                    GlobalValue::Resolving => {
                        // Note that the error we issue here isn't very informative (it doesn't include the actual cycle)
                        // but the general dependency cycle detector will give a better error later on during compilation.
                        let location = self.elaborator.interner.expr_location(&id);
                        Err(InterpreterError::GlobalsDependencyCycle { location })
                    }
                    GlobalValue::Unresolved => {
                        self.elaborator.interner.get_global_mut(global_id).value =
                            GlobalValue::Resolving;

                        self.elaborator.elaborate_global_if_unresolved(&global_id);

                        if let GlobalValue::Resolved(value) =
                            &self.elaborator.interner.get_global(global_id).value
                        {
                            Ok(value.clone())
                        } else {
                            let location = self.elaborator.interner.expr_location(&id);
                            Err(InterpreterError::GlobalCouldNotBeResolved { location })
                        }
                    }
                }
            }
            DefinitionKind::NumericGeneric(type_variable, numeric_typ) => {
                let value = Type::TypeVariable(type_variable.clone());
                self.evaluate_numeric_generic(value, numeric_typ, id)
            }
            DefinitionKind::AssociatedConstant(trait_impl_id, name) => {
                let associated_types =
                    self.elaborator.interner.get_associated_types_for_impl(*trait_impl_id);
                let associated_type = associated_types
                    .iter()
                    .find(|typ| typ.name.as_str() == name)
                    .expect("Expected to find associated type");

                let location = self.elaborator.interner.expr_location(&id);
                match associated_type
                    .typ
                    .evaluate_to_signed_field(&associated_type.typ.kind(), location)
                {
                    Ok(value) => self.evaluate_integer(value, id),
                    Err(err) => Err(InterpreterError::NonIntegerArrayLength {
                        typ: associated_type.typ.clone(),
                        err: Some(Box::new(err)),
                        location,
                    }),
                }
            }
        }
    }

    /// Evaluates a numeric generic with the value `value` (expected to be `Type::Constant`)
    /// and an expected integer type `expected`.
    fn evaluate_numeric_generic(&self, value: Type, expected: &Type, id: ExprId) -> IResult<Value> {
        let location = self.elaborator.interner.id_location(id);
        let value = value
            .evaluate_to_signed_field(&Kind::Numeric(Box::new(expected.clone())), location)
            .map_err(|err| {
                let typ = value;
                let err = Some(Box::new(err));
                let location = self.elaborator.interner.expr_location(&id);
                InterpreterError::NonIntegerArrayLength { typ, err, location }
            })?;

        self.evaluate_integer(value, id)
    }

    fn evaluate_trait_item(&mut self, item: TraitItem, id: ExprId) -> IResult<Value> {
        let typ = self.elaborator.interner.id_type(id).follow_bindings();
        match resolve_trait_item(self.elaborator.interner, item.id(), id)? {
            crate::monomorphization::TraitItem::Method(func_id) => {
                let bindings = self.elaborator.interner.get_instantiation_bindings(id).clone();
                Ok(Value::Function(func_id, typ, Rc::new(bindings)))
            }
            crate::monomorphization::TraitItem::Constant { id: _, expected_type, value } => {
                self.evaluate_numeric_generic(value, &expected_type, id)
            }
        }
    }

    fn evaluate_literal(&mut self, literal: HirLiteral, id: ExprId) -> IResult<Value> {
        match literal {
            HirLiteral::Unit => Ok(Value::Unit),
            HirLiteral::Bool(value) => Ok(Value::Bool(value)),
            HirLiteral::Integer(value) => self.evaluate_integer(value, id),
            HirLiteral::Str(string) => Ok(Value::String(Rc::new(string))),
            HirLiteral::FmtStr(fragments, captures, _length) => {
                self.evaluate_format_string(fragments, captures, id)
            }
            HirLiteral::Array(array) => self.evaluate_array(array, id),
            HirLiteral::Slice(array) => self.evaluate_slice(array, id),
        }
    }

    /// Evaluates a format string. Note that in doing so, the string is formatted now, there is no
    /// delayed formatting when it is later used. Effectively the result is identical to a normal
    /// string, just with a different type. This is also why when format strings are lowered into
    /// runtime code they become regular strings - because they're already formatted.
    fn evaluate_format_string(
        &mut self,
        fragments: Vec<FmtStrFragment>,
        captures: Vec<ExprId>,
        id: ExprId,
    ) -> IResult<Value> {
        let mut result = String::new();

        let mut values: VecDeque<_> =
            captures.into_iter().map(|capture| self.evaluate(capture)).collect::<Result<_, _>>()?;

        for fragment in fragments {
            match fragment {
                FmtStrFragment::String(string) => {
                    result.push_str(&string);
                }
                FmtStrFragment::Interpolation(..) => {
                    if let Some(value) = values.pop_front() {
                        // When interpolating a quoted value inside a format string, we don't include the
                        // surrounding `quote {` ... `}` as if we are unquoting the quoted value inside the string.
                        if let Value::Quoted(tokens) = value {
                            for (index, token) in tokens.iter().enumerate() {
                                if index > 0 {
                                    result.push(' ');
                                }
                                result.push_str(
                                    &token.token().display(self.elaborator.interner).to_string(),
                                );
                            }
                        } else {
                            result.push_str(&value.display(self.elaborator.interner).to_string());
                        }
                    } else {
                        // If we can't find a value for this fragment it means the interpolated value was not
                        // found or it errored. In this case we error here as well.
                        let location = self.elaborator.interner.expr_location(&id);
                        return Err(InterpreterError::CannotInterpretFormatStringWithErrors {
                            location,
                        });
                    }
                }
            }
        }

        let typ = self.elaborator.interner.id_type(id);
        Ok(Value::FormatString(Rc::new(result), typ))
    }

    /// Since integers are polymorphic, evaluating one requires the result type.
    /// We pass down the result type the elaborator previously inferred.
    fn evaluate_integer(&self, value: SignedField, id: ExprId) -> IResult<Value> {
        let typ = self.elaborator.interner.id_type(id).follow_bindings();
        let location = self.elaborator.interner.expr_location(&id);

        evaluate_integer(typ, value, location)
    }

    pub(crate) fn evaluate_block(&mut self, mut block: HirBlockExpression) -> IResult<Value> {
        let last_statement = block.statements.pop();
        self.push_scope();

        for statement in block.statements {
            let result = self.evaluate_statement(statement);
            if result.is_err() {
                self.pop_scope();
                return result;
            }
        }

        let result = if let Some(statement) = last_statement {
            self.evaluate_statement(statement)
        } else {
            Ok(Value::Unit)
        };

        self.pop_scope();
        result
    }

    fn evaluate_array(&mut self, array: HirArrayLiteral, id: ExprId) -> IResult<Value> {
        let typ = self.elaborator.interner.id_type(id).follow_bindings();

        match array {
            HirArrayLiteral::Standard(elements) => {
                let elements = elements
                    .into_iter()
                    .map(|id| self.evaluate(id))
                    .collect::<IResult<Vector<_>>>()?;

                Ok(Value::Array(elements, typ))
            }
            HirArrayLiteral::Repeated { repeated_element, length } => {
                let element = self.evaluate(repeated_element)?;

                let location = self.elaborator.interner.id_location(id);
                match length.evaluate_to_u32(location) {
                    Ok(length) => {
                        let elements = (0..length).map(|_| element.clone()).collect();
                        Ok(Value::Array(elements, typ))
                    }
                    Err(err) => {
                        let err = Some(Box::new(err));
                        let location = self.elaborator.interner.expr_location(&id);
                        Err(InterpreterError::NonIntegerArrayLength { typ: length, err, location })
                    }
                }
            }
        }
    }

    fn evaluate_slice(&mut self, array: HirArrayLiteral, id: ExprId) -> IResult<Value> {
        self.evaluate_array(array, id).map(|value| match value {
            Value::Array(array, typ) => Value::Slice(array, typ),
            other => unreachable!("Non-array value returned from evaluate array: {other:?}"),
        })
    }

    fn evaluate_prefix(&mut self, prefix: HirPrefixExpression, id: ExprId) -> IResult<Value> {
        let rhs = match prefix.operator {
            UnaryOp::Reference { .. } => self.evaluate_no_dereference(prefix.rhs)?,
            _ => self.evaluate(prefix.rhs)?,
        };

        if prefix.skip {
            return Ok(rhs);
        }

        if self.elaborator.interner.get_selected_impl_for_expression(id).is_some() {
            self.evaluate_overloaded_prefix(prefix, rhs, id)
        } else {
            let location = self.elaborator.interner.expr_location(&id);
            evaluate_prefix_with_value(rhs, prefix.operator, location)
        }
    }

    fn evaluate_infix(&mut self, infix: HirInfixExpression, id: ExprId) -> IResult<Value> {
        let lhs_value = self.evaluate(infix.lhs)?;
        let rhs_value = self.evaluate(infix.rhs)?;

        if self.elaborator.interner.get_selected_impl_for_expression(id).is_some() {
            return self.evaluate_overloaded_infix(infix, lhs_value, rhs_value, id);
        }

        let location = self.elaborator.interner.expr_location(&id);

        infix::evaluate_infix(lhs_value, rhs_value, infix.operator, location)
    }

    fn evaluate_overloaded_infix(
        &mut self,
        infix: HirInfixExpression,
        lhs: Value,
        rhs: Value,
        id: ExprId,
    ) -> IResult<Value> {
        let method = infix.trait_method_id;
        let operator = infix.operator.kind;

        let method_id = resolve_trait_item(self.elaborator.interner, method, id)?.unwrap_method();
        let type_bindings = self.elaborator.interner.get_instantiation_bindings(id).clone();

        let lhs = (lhs, self.elaborator.interner.expr_location(&infix.lhs));
        let rhs = (rhs, self.elaborator.interner.expr_location(&infix.rhs));

        let location = self.elaborator.interner.expr_location(&id);
        let value = self.call_function(method_id, vec![lhs, rhs], type_bindings, location)?;

        // Certain operators add additional operations after the trait call:
        // - `!=`: Reverse the result of Eq
        // - Comparator operators: Convert the returned `Ordering` to a boolean.
        use BinaryOpKind::*;
        match operator {
            NotEqual => evaluate_prefix_with_value(value, UnaryOp::Not, location),
            Less | LessEqual | Greater | GreaterEqual => {
                self.evaluate_ordering(value, operator, location)
            }
            _ => Ok(value),
        }
    }

    fn evaluate_overloaded_prefix(
        &mut self,
        prefix: HirPrefixExpression,
        rhs: Value,
        id: ExprId,
    ) -> IResult<Value> {
        let method =
            prefix.trait_method_id.expect("ice: expected prefix operator trait at this point");

        let method_id = resolve_trait_item(self.elaborator.interner, method, id)?.unwrap_method();
        let type_bindings = self.elaborator.interner.get_instantiation_bindings(id).clone();

        let rhs = (rhs, self.elaborator.interner.expr_location(&prefix.rhs));

        let location = self.elaborator.interner.expr_location(&id);
        self.call_function(method_id, vec![rhs], type_bindings, location)
    }

    /// Given the result of a `cmp` operation, convert it into the boolean result of the given operator.
    fn evaluate_ordering(
        &self,
        ordering: Value,
        operator: BinaryOpKind,
        location: Location,
    ) -> IResult<Value> {
        let field_ordering = match &ordering {
            Value::Struct(fields, typ) => {
                // Check the struct is named "Ordering"
                let is_ordering_type = match typ.follow_bindings() {
                    Type::DataType(def, _) => def.borrow().name.as_str() == "Ordering",
                    _ => false,
                };
                if is_ordering_type {
                    let first_field = fields.iter().next();
                    match first_field {
                        Some((_, value)) => match &*value.borrow() {
                            Value::Field(ordering) => Some(*ordering),
                            _ => None,
                        },
                        None => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        // Error if there is no ordering field
        let Some(ordering) = field_ordering else {
            return Err(InterpreterError::TypeMismatch {
                expected: "Ordering".to_string(),
                actual: ordering.get_type().into_owned(),
                location,
            });
        };

        // Ordering::Less: 0, Ordering::Equal: 1, Ordering::Greater: 2
        let result = match operator {
            // `<`:  `ordering == Ordering::Less`
            BinaryOpKind::Less => ordering.is_zero(),
            // `<=`: `ordering != Ordering::Greater`
            BinaryOpKind::LessEqual => ordering != SignedField::positive(2_u128),
            // `>`:  `ordering == Ordering::Greater`
            BinaryOpKind::Greater => ordering == SignedField::positive(2_u128),
            // `>=`: `ordering != Ordering::Less`
            BinaryOpKind::GreaterEqual => !ordering.is_zero(),
            _ => unreachable!("evaluate_ordering called with non-ordering operator"),
        };
        Ok(Value::Bool(result))
    }

    fn evaluate_index(&mut self, index: HirIndexExpression, id: ExprId) -> IResult<Value> {
        let idx = self.evaluate(index.index)?;
        let array = self.evaluate(index.collection)?;

        let location = self.elaborator.interner.expr_location(&id);
        let (array, idx) = bounds_check(array, idx, location)?;

        Ok(array[idx].clone())
    }

    fn evaluate_constructor(
        &mut self,
        constructor: HirConstructorExpression,
        id: ExprId,
    ) -> IResult<Value> {
        let fields = constructor
            .fields
            .into_iter()
            .map(|(name, expr)| {
                let field_value = Shared::new(self.evaluate(expr)?);
                Ok((Rc::new(name.into_string()), field_value))
            })
            .collect::<Result<_, _>>()?;

        let typ = self.elaborator.interner.id_type(id).follow_bindings();
        Ok(Value::Struct(fields, typ))
    }

    /// Unlike a struct constructor, an enum constructor inserts a tag value along with the fields
    fn evaluate_enum_constructor(
        &mut self,
        constructor: HirEnumConstructorExpression,
        id: ExprId,
    ) -> IResult<Value> {
        let fields = try_vecmap(constructor.arguments, |arg| self.evaluate(arg))?;
        let typ = self.elaborator.interner.id_type(id).unwrap_forall().1.follow_bindings();
        Ok(Value::Enum(constructor.variant_index, fields, typ))
    }

    fn evaluate_access(&mut self, access: HirMemberAccess, id: ExprId) -> IResult<Value> {
        let lhs = self.evaluate_no_dereference(access.lhs)?;
        let is_offset = access.is_offset && lhs.get_type().is_ref();

        let field = self.get_field(lhs, id, access.rhs.as_string())?;

        // Return a reference to the field so that `&mut foo.bar.baz` can use this reference.
        // We set auto_deref to true so that when it is used elsewhere it is dereferenced
        // automatically. In some cases in the frontend the leading `&mut` will cancel out
        // with a field access which is expected to only offset into the struct and thus return
        // a reference already. In this case we set auto_deref to false because the outer `&mut`
        // will also be removed in that case so the pointer should be explicit.
        let auto_deref = !is_offset;
        Ok(Value::Pointer(field, auto_deref, false))
    }

    /// Given a value, return the struct/tuple field with the given name, automatically dereferencing any
    /// pointers found.
    fn get_field(&mut self, value: Value, id: ExprId, name: &String) -> IResult<Shared<Value>> {
        let typ = match value {
            Value::Struct(fields, struct_type) => match fields.get(name) {
                Some(field) => return Ok(field.clone()),
                None => struct_type,
            },
            Value::Tuple(types) => {
                let index = name.parse::<usize>().ok();
                match index.and_then(|index| types.get(index)) {
                    Some(value) => return Ok(value.clone()),
                    None => Type::Tuple(vecmap(types, |typ| typ.borrow().get_type().into_owned())),
                }
            }
            Value::Pointer(element, ..) => {
                return self.get_field(element.unwrap_or_clone(), id, name);
            }
            value => {
                let location = self.elaborator.interner.expr_location(&id);
                let typ = value.get_type().into_owned();
                return Err(InterpreterError::NonTupleOrStructInMemberAccess { typ, location });
            }
        };

        let location = self.elaborator.interner.expr_location(&id);
        let field_name = name.to_string();
        Err(InterpreterError::ExpectedStructToHaveField { typ, field_name, location })
    }

    /// Evaluates a call expression, deferring to [Self::call_function] or [Self::call_closure]
    /// once the function is determined.
    fn evaluate_call(&mut self, call: HirCallExpression, id: ExprId) -> IResult<Value> {
        let function = self.evaluate(call.func)?;
        let arguments = try_vecmap(call.arguments, |arg| {
            Ok((self.evaluate(arg)?, self.elaborator.interner.expr_location(&arg)))
        })?;
        let location = self.elaborator.interner.expr_location(&id);

        match function {
            Value::Function(function_id, _, bindings) => {
                let bindings = unwrap_rc(bindings);
                let mut result = self.call_function(function_id, arguments, bindings, location)?;
                if call.is_macro_call {
                    let expr = result.into_expression(self.elaborator, location)?;
                    let expr =
                        self.elaborate_in_function(self.current_function, None, |elaborator| {
                            elaborator.elaborate_expression(expr).0
                        });
                    result = self.evaluate(expr)?;

                    // Macro calls are typed as type variables during type checking.
                    // Now that we know the type we need to further unify it in case there
                    // are inconsistencies or the type needs to be known.
                    // We don't commit any type bindings made this way in case the type of
                    // the macro result changes across loop iterations.
                    let expected_type = self.elaborator.interner.id_type(id);
                    let actual_type = result.get_type();
                    self.unify_without_binding(&actual_type, &expected_type, location);
                }
                Ok(result)
            }
            Value::Closure(closure) => self.call_closure(
                closure.lambda,
                closure.env,
                arguments,
                closure.function_scope,
                closure.module_scope,
                location,
            ),
            value => {
                let typ = value.get_type().into_owned();
                Err(InterpreterError::NonFunctionCalled { typ, location })
            }
        }
    }

    /// This function is used by the interpreter for some comptime code
    /// which can change types e.g. on each iteration of a for loop.
    fn unify_without_binding(&mut self, actual: &Type, expected: &Type, location: Location) {
        let mut bindings = TypeBindings::default();
        if actual.try_unify(expected, &mut bindings).is_err() {
            let error = TypeCheckError::TypeMismatch {
                expected_typ: expected.to_string(),
                expr_typ: actual.to_string(),
                expr_location: location,
            };
            self.elaborator.push_err(error);
        }
    }

    fn evaluate_cast(&mut self, cast: &HirCastExpression, id: ExprId) -> IResult<Value> {
        let evaluated_lhs = self.evaluate(cast.lhs)?;
        let location = self.elaborator.interner.expr_location(&id);
        cast::evaluate_cast_one_step(&cast.r#type, location, evaluated_lhs)
    }

    fn evaluate_if(&mut self, if_: HirIfExpression) -> IResult<Value> {
        let condition = match self.evaluate(if_.condition)? {
            Value::Bool(value) => value,
            value => {
                let location = self.elaborator.interner.expr_location(&if_.condition);
                let typ = value.get_type().into_owned();
                return Err(InterpreterError::NonBoolUsedInIf { typ, location });
            }
        };

        self.push_scope();

        let result = if condition {
            if if_.alternative.is_some() {
                self.evaluate(if_.consequence)
            } else {
                let result = self.evaluate(if_.consequence);
                if result.is_err() {
                    self.pop_scope();
                    return result;
                }
                Ok(Value::Unit)
            }
        } else {
            match if_.alternative {
                Some(alternative) => self.evaluate(alternative),
                None => Ok(Value::Unit),
            }
        };

        self.pop_scope();
        result
    }

    fn evaluate_tuple(&mut self, tuple: Vec<ExprId>) -> IResult<Value> {
        let fields = try_vecmap(tuple, |field| Ok(Shared::new(self.evaluate(field)?)))?;
        Ok(Value::Tuple(fields))
    }

    fn evaluate_lambda(&mut self, lambda: HirLambda, id: ExprId) -> IResult<Value> {
        let location = self.elaborator.interner.expr_location(&id);
        let env = try_vecmap(&lambda.captures, |capture| {
            let value = self.lookup_id(capture.ident.id, location)?;
            match value {
                // Dereference mutable variables to capture by value
                Value::Pointer(elem, true, _) => Ok(elem.unwrap_or_clone()),
                other => Ok(other),
            }
        })?;

        let typ = self.elaborator.interner.id_type(id).follow_bindings();
        let module_scope = self.elaborator.module_id();
        let closure =
            Closure { lambda, env, typ, function_scope: self.current_function, module_scope };
        Ok(Value::Closure(Box::new(closure)))
    }

    fn evaluate_quote(&mut self, tokens: Tokens) -> IResult<Value> {
        let tokens = self.substitute_unquoted_values_into_tokens(tokens)?;
        Ok(Value::Quoted(Rc::new(tokens)))
    }

    pub fn evaluate_statement(&mut self, statement: StmtId) -> IResult<Value> {
        // If comptime evaluation has been halted, don't execute anything
        if self.elaborator.comptime_evaluation_halted {
            return Err(InterpreterError::SkippedDueToEarlierErrors);
        }

        // Skip statements that had errors during elaboration and halt all future execution
        if self.elaborator.interner.stmts_with_errors.contains(&statement) {
            self.elaborator.comptime_evaluation_halted = true;
            return Err(InterpreterError::SkippedDueToEarlierErrors);
        }

        match self.elaborator.interner.statement(&statement) {
            HirStatement::Let(let_) => self.evaluate_let(let_),
            HirStatement::Assign(assign) => self.evaluate_assign(assign),
            HirStatement::For(for_) => self.evaluate_for(for_),
            HirStatement::Loop(expression) => self.evaluate_loop(expression),
            HirStatement::While(condition, block) => self.evaluate_while(condition, block),
            HirStatement::Break => self.evaluate_break(statement),
            HirStatement::Continue => self.evaluate_continue(statement),
            HirStatement::Expression(expression) => self.evaluate(expression),
            HirStatement::Comptime(statement) => self.evaluate_comptime(statement),
            HirStatement::Semi(expression) => {
                self.evaluate(expression)?;
                Ok(Value::Unit)
            }
            HirStatement::Error => {
                let location = self.elaborator.interner.id_location(statement);
                Err(InterpreterError::ErrorNodeEncountered { location })
            }
        }
    }

    pub(crate) fn evaluate_let(&mut self, let_: HirLetStatement) -> IResult<Value> {
        let rhs = self.evaluate(let_.expression)?;
        let location = self.elaborator.interner.expr_location(&let_.expression);
        self.define_pattern(&let_.pattern, &let_.r#type, rhs, location)?;
        Ok(Value::Unit)
    }

    fn evaluate_constrain(&mut self, constrain: HirConstrainExpression) -> IResult<Value> {
        match self.evaluate(constrain.0)? {
            Value::Bool(true) => Ok(Value::Unit),
            Value::Bool(false) => {
                let location = self.elaborator.interner.expr_location(&constrain.0);
                let message = constrain.2.and_then(|expr| self.evaluate(expr).ok());
                let message =
                    message.map(|value| value.display(self.elaborator.interner).to_string());
                let call_stack = self.elaborator.interpreter_call_stack().clone();
                Err(InterpreterError::FailingConstraint { location, message, call_stack })
            }
            value => {
                let location = self.elaborator.interner.expr_location(&constrain.0);
                let typ = value.get_type().into_owned();
                Err(InterpreterError::NonBoolUsedInConstrain { typ, location })
            }
        }
    }

    fn evaluate_assign(&mut self, assign: HirAssignStatement) -> IResult<Value> {
        let rhs = self.evaluate(assign.expression)?;
        self.store_lvalue(assign.lvalue, rhs)?;
        Ok(Value::Unit)
    }

    /// Stores `rhs` at the location determined by `lvalue`
    fn store_lvalue(&mut self, lvalue: HirLValue, rhs: Value) -> IResult<()> {
        match lvalue {
            HirLValue::Ident(ident, _typ) => self.mutate(ident.id, rhs, ident.location),
            HirLValue::Dereference { lvalue, element_type: _, location, implicitly_added: _ } => {
                match self.evaluate_lvalue(&lvalue)? {
                    Value::Pointer(value, _, _) => {
                        Self::store_flattened(value, rhs);
                        Ok(())
                    }
                    value => {
                        let typ = value.get_type().into_owned();
                        Err(InterpreterError::NonPointerDereferenced { typ, location })
                    }
                }
            }
            HirLValue::MemberAccess { object, field_name, field_index, typ: _, location } => {
                let object_value = self.evaluate_lvalue(&object)?;

                let index = field_index.ok_or_else(|| {
                    let value = object_value.clone();
                    let field_name = field_name.to_string();
                    let typ = value.get_type().into_owned();
                    InterpreterError::ExpectedStructToHaveField { typ, field_name, location }
                })?;

                match object_value {
                    Value::Tuple(mut fields) => {
                        fields[index] = Shared::new(rhs);
                        self.store_lvalue(*object, Value::Tuple(fields))
                    }
                    Value::Struct(mut fields, typ) => {
                        fields.insert(Rc::new(field_name.into_string()), Shared::new(rhs));
                        self.store_lvalue(*object, Value::Struct(fields, typ.follow_bindings()))
                    }
                    value => {
                        let typ = value.get_type().into_owned();
                        Err(InterpreterError::NonTupleOrStructInMemberAccess { typ, location })
                    }
                }
            }
            HirLValue::Index { array, index, typ: _, location } => {
                let array_value = self.evaluate_lvalue(&array)?;
                let index = self.evaluate(index)?;

                let constructor = match &array_value {
                    Value::Array(..) => Value::Array,
                    _ => Value::Slice,
                };

                let typ = array_value.get_type().into_owned();
                let (elements, index) = bounds_check(array_value, index, location)?;

                let new_array = constructor(elements.update(index, rhs), typ);
                self.store_lvalue(*array, new_array)
            }
        }
    }

    /// When we store to a struct such as in
    /// ```noir
    /// let mut a = (false,);
    /// let b = &mut a.0;
    /// a = (true,);
    /// ```
    /// we must flatten the store to store to each individual field so that any existing
    /// references, such as `b` above, will also reflect the mutation.
    fn store_flattened(lvalue: Shared<Value>, rvalue: Value) {
        let lvalue_ref = lvalue.borrow();
        match (&*lvalue_ref, rvalue) {
            (Value::Struct(lvalue_fields, _), Value::Struct(mut rvalue_fields, _)) => {
                for (name, lvalue) in lvalue_fields.iter() {
                    let Some(rvalue) = rvalue_fields.remove(name) else { continue };
                    Self::store_flattened(lvalue.clone(), rvalue.unwrap_or_clone());
                }
            }
            (Value::Tuple(lvalue_fields), Value::Tuple(rvalue_fields)) => {
                for (lvalue, rvalue) in lvalue_fields.iter().zip(rvalue_fields) {
                    Self::store_flattened(lvalue.clone(), rvalue.unwrap_or_clone());
                }
            }
            (_, rvalue) => {
                drop(lvalue_ref);
                *lvalue.borrow_mut() = rvalue;
            }
        }
    }

    /// Returns the current value held by `lvalue`
    fn evaluate_lvalue(&mut self, lvalue: &HirLValue) -> IResult<Value> {
        match lvalue {
            HirLValue::Ident(ident, _) => match self.lookup(ident)? {
                Value::Pointer(elem, true, _) => Ok(elem.borrow().clone()),
                other => Ok(other),
            },
            HirLValue::Dereference { lvalue, element_type: _, location, implicitly_added: _ } => {
                match self.evaluate_lvalue(lvalue)? {
                    Value::Pointer(value, _, _) => Ok(value.borrow().clone()),
                    value => {
                        let typ = value.get_type().into_owned();
                        Err(InterpreterError::NonPointerDereferenced { typ, location: *location })
                    }
                }
            }
            HirLValue::MemberAccess { object, field_name, field_index, typ: _, location } => {
                let object_value = self.evaluate_lvalue(object)?;

                let index = field_index.ok_or_else(|| {
                    let value = object_value.clone();
                    let field_name = field_name.to_string();
                    let location = *location;
                    let typ = value.get_type().into_owned();
                    InterpreterError::ExpectedStructToHaveField { typ, field_name, location }
                })?;

                match object_value {
                    Value::Tuple(mut values) => Ok(values.swap_remove(index).unwrap_or_clone()),
                    Value::Struct(fields, _) => {
                        Ok(fields[field_name.as_string()].clone().unwrap_or_clone())
                    }
                    value => Err(InterpreterError::NonTupleOrStructInMemberAccess {
                        typ: value.get_type().into_owned(),
                        location: *location,
                    }),
                }
            }
            HirLValue::Index { array, index, typ: _, location } => {
                let array = self.evaluate_lvalue(array)?;
                let index = self.evaluate(*index)?;
                let (elements, index) = bounds_check(array, index, *location)?;
                Ok(elements[index].clone())
            }
        }
    }

    fn evaluate_for(&mut self, for_: HirForStatement) -> IResult<Value> {
        let start_value = self.evaluate(for_.start_range)?;
        let end_value = self.evaluate(for_.end_range)?;
        let start_type = start_value.get_type();
        let end_type = end_value.get_type();

        // Check that start and end have the same type
        if start_type.unify(&end_type).is_err() {
            let location = self.elaborator.interner.expr_location(&for_.end_range);
            return Err(InterpreterError::RangeBoundsTypeMismatch {
                start_type: start_type.into_owned(),
                end_type: end_type.into_owned(),
                location,
            });
        }

        if start_type.is_signed() {
            let get_index = match start_value {
                Value::I8(_) => |i| Value::I8(i as i8),
                Value::I16(_) => |i| Value::I16(i as i16),
                Value::I32(_) => |i| Value::I32(i as i32),
                Value::I64(_) => |i| Value::I64(i as i64),
                _ => unreachable!("Checked above that value is signed type"),
            };

            // i128 can store all values from i8 - u64
            let start = to_i128(start_value).expect("Checked above that value is signed type");
            let end = to_i128(end_value).expect("Checked above that types match");

            self.evaluate_for_loop(start..end, get_index, for_.identifier.id, for_.block)
        } else if start_type.is_unsigned() {
            let get_index = match start_value {
                Value::U1(_) => |i| Value::U1(i == 1),
                Value::U8(_) => |i| Value::U8(i as u8),
                Value::U16(_) => |i| Value::U16(i as u16),
                Value::U32(_) => |i| Value::U32(i as u32),
                Value::U64(_) => |i| Value::U64(i as u64),
                Value::U128(_) => |i| Value::U128(i),
                _ => unreachable!("Checked above that value is unsigned type"),
            };

            // u128 can store all values from u8 - u128
            let start = to_u128(start_value).expect("Checked above that value is unsigned type");
            let end = to_u128(end_value).expect("Checked above that types match");

            self.evaluate_for_loop(start..end, get_index, for_.identifier.id, for_.block)
        } else {
            let location = self.elaborator.interner.expr_location(&for_.start_range);
            let typ = start_type.into_owned();
            Err(InterpreterError::NonIntegerUsedInLoop { typ, location })
        }
    }

    fn evaluate_for_loop<T>(
        &mut self,
        range_iterator: impl Iterator<Item = T>,
        get_index: fn(T) -> Value,
        index_id: DefinitionId,
        block: ExprId,
    ) -> IResult<Value> {
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);

        let mut result = Ok(Value::Unit);

        for i in range_iterator {
            self.push_scope();
            self.current_scope_mut().insert(index_id, get_index(i));

            let must_break = self.evaluate_loop_body(block, &mut result);

            self.pop_scope();

            if must_break {
                break;
            }
        }

        self.in_loop = was_in_loop;
        result
    }

    fn evaluate_loop(&mut self, expr: ExprId) -> IResult<Value> {
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);
        let in_lsp = self.elaborator.interner.is_in_lsp_mode();
        let mut counter = 0;
        let mut result = Ok(Value::Unit);

        loop {
            self.push_scope();

            let must_break = self.evaluate_loop_body(expr, &mut result);

            self.pop_scope();

            if must_break {
                break;
            }

            counter += 1;
            if in_lsp && counter == 10_000 {
                let location = self.elaborator.interner.expr_location(&expr);
                result = Err(InterpreterError::LoopHaltedForUiResponsiveness { location });
                break;
            }
        }

        self.in_loop = was_in_loop;
        result
    }

    fn evaluate_while(&mut self, condition: ExprId, block: ExprId) -> IResult<Value> {
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);
        let in_lsp = self.elaborator.interner.is_in_lsp_mode();
        let mut counter = 0;
        let mut result = Ok(Value::Unit);

        loop {
            let condition = match self.evaluate(condition) {
                Ok(Value::Bool(value)) => value,
                Ok(value) => {
                    let location = self.elaborator.interner.expr_location(&condition);
                    let typ = value.get_type().into_owned();
                    result = Err(InterpreterError::NonBoolUsedInWhile { typ, location });
                    break;
                }
                Err(err) => {
                    result = Err(err);
                    break;
                }
            };

            if !condition {
                break;
            }

            self.push_scope();

            let must_break = self.evaluate_loop_body(block, &mut result);
            self.pop_scope();

            if must_break {
                break;
            }

            counter += 1;
            if in_lsp && counter == 10_000 {
                let location = self.elaborator.interner.expr_location(&block);
                result = Err(InterpreterError::LoopHaltedForUiResponsiveness { location });
                break;
            }
        }

        self.in_loop = was_in_loop;
        result
    }

    /// Evaluate one iteration of a loop.
    ///
    /// Returns a flag to indicate whether the loop should be exited.
    fn evaluate_loop_body(&mut self, body: ExprId, result: &mut IResult<Value>) -> bool {
        match self.evaluate(body) {
            Ok(_) => false,
            Err(InterpreterError::Break) => true,
            Err(InterpreterError::Continue) => false,
            Err(error) => {
                *result = Err(error);
                true
            }
        }
    }

    fn evaluate_break(&mut self, id: StmtId) -> IResult<Value> {
        if self.in_loop {
            Err(InterpreterError::Break)
        } else {
            let location = self.elaborator.interner.statement_location(id);
            Err(InterpreterError::BreakNotInLoop { location })
        }
    }

    fn evaluate_continue(&mut self, id: StmtId) -> IResult<Value> {
        if self.in_loop {
            Err(InterpreterError::Continue)
        } else {
            let location = self.elaborator.interner.statement_location(id);
            Err(InterpreterError::ContinueNotInLoop { location })
        }
    }

    pub(super) fn evaluate_comptime(&mut self, statement: StmtId) -> IResult<Value> {
        self.evaluate_statement(statement)
    }

    fn print_oracle(
        &mut self,
        arguments: Vec<(Value, Location)>,
    ) -> Result<Value, InterpreterError> {
        assert_eq!(arguments.len(), 2);

        let Some(output) = self.elaborator.interpreter_output else {
            return Ok(Value::Unit);
        };

        let mut output = output.borrow_mut();

        let print_newline = arguments[0].0 == Value::Bool(true);
        let contents = arguments[1].0.display(self.elaborator.interner);
        if self.elaborator.interner.is_in_lsp_mode() {
            // If we `println!` in LSP it gets mixed with the protocol stream and leads to crashing
            // the connection. If we use `eprintln!` not only it doesn't crash, but the output
            // appears in the "Noir Language Server" output window in case you want to see it.
            if print_newline {
                eprintln!("{contents}");
            } else {
                eprint!("{contents}");
            }
        } else if print_newline {
            writeln!(output, "{contents}").expect("write should succeed");
        } else {
            write!(output, "{contents}").expect("write should succeed");
        }

        Ok(Value::Unit)
    }
}

fn evaluate_integer(typ: Type, value: SignedField, location: Location) -> IResult<Value> {
    use IntegerBitSize::*;
    use Signedness::*;
    use Type::*;

    macro_rules! evaluate_unsigned {
        ($typ:ident) => {{
            let value = value
                .try_to_unsigned()
                .ok_or(InterpreterError::IntegerOutOfRangeForType { value, typ, location })?;
            Ok(Value::$typ(value))
        }};
    }

    macro_rules! evaluate_signed {
        ($typ:ident) => {{
            let value = value
                .try_to_signed()
                .ok_or(InterpreterError::IntegerOutOfRangeForType { value, typ, location })?;
            Ok(Value::$typ(value))
        }};
    }

    match typ {
        FieldElement => Ok(Value::Field(value)),
        Integer(Unsigned, One) => {
            let field_value = value.to_field_element();
            if field_value.is_zero() {
                Ok(Value::U1(false))
            } else if field_value.is_one() {
                Ok(Value::U1(true))
            } else {
                Err(InterpreterError::IntegerOutOfRangeForType { value, typ, location })
            }
        }
        Integer(Unsigned, Eight) => {
            evaluate_unsigned!(U8)
        }
        Integer(Unsigned, Sixteen) => {
            evaluate_unsigned!(U16)
        }
        Integer(Unsigned, ThirtyTwo) => {
            evaluate_unsigned!(U32)
        }
        Integer(Unsigned, SixtyFour) => {
            evaluate_unsigned!(U64)
        }
        Integer(Unsigned, HundredTwentyEight) => {
            evaluate_unsigned!(U128)
        }
        Integer(Signed, One) => Err(InterpreterError::TypeUnsupported { typ, location }),
        Integer(Signed, Eight) => {
            evaluate_signed!(I8)
        }
        Integer(Signed, Sixteen) => {
            evaluate_signed!(I16)
        }
        Integer(Signed, ThirtyTwo) => {
            evaluate_signed!(I32)
        }
        Integer(Signed, SixtyFour) => {
            evaluate_signed!(I64)
        }
        Integer(Signed, HundredTwentyEight) => {
            Err(InterpreterError::TypeUnsupported { typ, location })
        }
        _ => Err(InterpreterError::NonIntegerIntegerLiteral { typ, location }),
    }
}

/// Bounds check the given array and index pair.
/// This will also ensure the given arguments are in fact an array and integer.
fn bounds_check(array: Value, index: Value, location: Location) -> IResult<(Vector<Value>, usize)> {
    let collection = match array {
        Value::Array(array, _) => array,
        Value::Slice(array, _) => array,
        value => {
            let typ = value.get_type().into_owned();
            return Err(InterpreterError::NonArrayIndexed { typ, location });
        }
    };

    let index = match index {
        Value::Field(value) => {
            let u64: Option<u64> = value.try_to_unsigned();
            u64.and_then(|value| value.try_into().ok()).ok_or_else(|| {
                let typ = Type::default_int_type();
                InterpreterError::IntegerOutOfRangeForType { value, typ, location }
            })?
        }
        Value::I8(value) => value as usize,
        Value::I16(value) => value as usize,
        Value::I32(value) => value as usize,
        Value::I64(value) => value as usize,
        Value::U1(value) => {
            if value {
                1_usize
            } else {
                0_usize
            }
        }
        Value::U8(value) => value as usize,
        Value::U16(value) => value as usize,
        Value::U32(value) => value as usize,
        Value::U64(value) => value as usize,
        Value::U128(value) => value as usize,
        value => {
            let typ = value.get_type().into_owned();
            return Err(InterpreterError::NonIntegerUsedAsIndex { typ, location });
        }
    };

    if index >= collection.len() {
        use InterpreterError::IndexOutOfBounds;
        return Err(IndexOutOfBounds { index, location, length: collection.len() });
    }

    Ok((collection, index))
}

fn evaluate_prefix_with_value(rhs: Value, operator: UnaryOp, location: Location) -> IResult<Value> {
    match operator {
        UnaryOp::Minus => match rhs {
            Value::Field(value) => Ok(Value::Field(-value)),
            Value::I8(value) => value
                .checked_neg()
                .map(Value::I8)
                .ok_or_else(|| InterpreterError::NegateWithOverflow { location }),
            Value::I16(value) => value
                .checked_neg()
                .map(Value::I16)
                .ok_or_else(|| InterpreterError::NegateWithOverflow { location }),
            Value::I32(value) => value
                .checked_neg()
                .map(Value::I32)
                .ok_or_else(|| InterpreterError::NegateWithOverflow { location }),
            Value::I64(value) => value
                .checked_neg()
                .map(Value::I64)
                .ok_or_else(|| InterpreterError::NegateWithOverflow { location }),
            Value::U1(_) => Err(InterpreterError::CannotApplyMinusToType { location, typ: "u1" }),
            Value::U8(_) => Err(InterpreterError::CannotApplyMinusToType { location, typ: "u8" }),
            Value::U16(_) => Err(InterpreterError::CannotApplyMinusToType { location, typ: "u16" }),
            Value::U32(_) => Err(InterpreterError::CannotApplyMinusToType { location, typ: "u32" }),
            Value::U64(_) => Err(InterpreterError::CannotApplyMinusToType { location, typ: "u64" }),
            Value::U128(_) => {
                Err(InterpreterError::CannotApplyMinusToType { location, typ: "u128" })
            }
            value => {
                let operator = "minus";
                let typ = value.get_type().into_owned();
                Err(InterpreterError::InvalidValueForUnary { typ, location, operator })
            }
        },
        UnaryOp::Not => match rhs {
            Value::Bool(value) => Ok(Value::Bool(!value)),
            Value::I8(value) => Ok(Value::I8(!value)),
            Value::I16(value) => Ok(Value::I16(!value)),
            Value::I32(value) => Ok(Value::I32(!value)),
            Value::I64(value) => Ok(Value::I64(!value)),
            Value::U1(value) => Ok(Value::U1(!value)),
            Value::U8(value) => Ok(Value::U8(!value)),
            Value::U16(value) => Ok(Value::U16(!value)),
            Value::U32(value) => Ok(Value::U32(!value)),
            Value::U64(value) => Ok(Value::U64(!value)),
            Value::U128(value) => Ok(Value::U128(!value)),
            value => {
                let typ = value.get_type().into_owned();
                Err(InterpreterError::InvalidValueForUnary { typ, location, operator: "not" })
            }
        },
        UnaryOp::Reference { mutable } => {
            // If this is a mutable variable (auto_deref = true), turn this into an explicit
            // mutable reference just by switching the value of `auto_deref`. Otherwise, wrap
            // the value in a fresh reference.
            match rhs {
                Value::Pointer(elem, true, _) => Ok(Value::Pointer(elem, false, mutable)),
                other => Ok(Value::Pointer(Shared::new(other), false, mutable)),
            }
        }
        UnaryOp::Dereference { implicitly_added: _ } => match rhs {
            Value::Pointer(element, _, _) => Ok(element.borrow().clone()),
            value => {
                let typ = value.get_type().into_owned();
                Err(InterpreterError::NonPointerDereferenced { typ, location })
            }
        },
    }
}

fn to_u128(value: Value) -> Option<u128> {
    match value {
        Value::U1(value) => Some(u128::from(value)),
        Value::U8(value) => Some(u128::from(value)),
        Value::U16(value) => Some(u128::from(value)),
        Value::U32(value) => Some(u128::from(value)),
        Value::U64(value) => Some(u128::from(value)),
        Value::U128(value) => Some(value),
        _ => None,
    }
}

fn to_i128(value: Value) -> Option<i128> {
    match value {
        Value::I8(value) => Some(i128::from(value)),
        Value::I16(value) => Some(i128::from(value)),
        Value::I32(value) => Some(i128::from(value)),
        Value::I64(value) => Some(i128::from(value)),
        _ => None,
    }
}

impl Context<'_, '_> {
    /// Interprets (as comptime code) the given function in the give crate, with the given arguments.
    /// Panics if there's no main function.
    pub fn interpret_function(
        &mut self,
        main_id: FuncId,
        args: Vec<(Value, Location)>,
    ) -> IResult<Value> {
        let func_meta = self.def_interner.function_meta(&main_id);
        let crate_id = func_meta.source_crate;
        let local_id = func_meta.source_module;
        let location = func_meta.location;
        let enabled_unstable_features = &self.required_unstable_features[&crate_id].clone();
        let cli_options = ElaboratorOptions {
            debug_comptime_in_file: None,
            pedantic_solving: false,
            enabled_unstable_features,
            disable_required_unstable_features: false,
        };
        let module_id = ModuleId { krate: crate_id, local_id };

        let mut elaborator = Elaborator::from_context(self, crate_id, cli_options);
        elaborator.replace_module(module_id);

        let mut interpreter = elaborator.setup_interpreter();
        let instantiation_bindings = TypeBindings::default();
        interpreter.call_function(main_id, args, instantiation_bindings, location)
    }
}
