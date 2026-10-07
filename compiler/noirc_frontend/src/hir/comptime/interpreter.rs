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
//! [`InterpreterError::ArgumentCountMismatch`] is an example of such an error.

use std::collections::VecDeque;
use std::rc::Rc;

use acvm::AcirField;
use imbl::Vector;
use iter_extended::{try_vecmap, vecmap};
use itertools::Itertools;
use noirc_errors::Location;
use num_bigint::BigInt;
use rustc_hash::FxHashMap as HashMap;

use crate::ast::{BinaryOpKind, FunctionKind, IntegerBitSize, UnaryOp};
use crate::elaborator::{Elaborator, ElaboratorOptions};
use crate::hir::Context;
use crate::hir::comptime::Integer;
use crate::hir::comptime::ValueCell;
use crate::hir::comptime::value::FormatStringFragment;
use crate::hir::def_map::ModuleId;
use crate::hir_def::types::resolve_type_bindings;
use crate::monomorphization::{compute_impl_bindings, resolve_trait_item};
use crate::node_interner::GlobalValue;
use crate::shared::{Builtin, ForeignCall, Signedness};
use crate::token::{FmtStrFragment, Tokens};
use crate::{
    Type, TypeBindings,
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
    node_interner::{DefinitionId, DefinitionKind, ExprId, FuncId, StmtId, TraitItemId},
};
use crate::{TypeVariableId, UnificationError};

use super::errors::{IResult, InterpreterError};
use super::value::{Closure, Value, unwrap_rc};

mod builtin;
mod cast;
pub(crate) use cast::evaluate_cast_one_step;
mod foreign;
mod frame;
pub(crate) use frame::Frame;
mod infix;
mod tracker;
pub use tracker::EvaluationTracker;
mod unquote;

pub(crate) use builtin::builtin_helpers;

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

    /// The types the function being interpreted sees. Every type this interpreter reads from the
    /// HIR of that function goes through [`Self::ty`], which applies them.
    frame: Frame,

    /// The type variables each call expression solved in [`Self::frame`] the last time it was
    /// evaluated: from its result, or handed back by its callee. A call in a loop can solve them
    /// differently on each iteration, so they are taken back out before it is evaluated again.
    call_bindings: HashMap<ExprId, Vec<TypeVariableId>>,

    /// Current evaluation depth.
    evaluation_depth: usize,

    /// Whether we are inside an unconstrained context. This is set to true when entering
    /// an unconstrained function, and it keeps being true in nested calls regardless of
    /// them being constrained or unconstrained.
    in_unconstrained: bool,
}

impl<'local, 'interner> Interpreter<'local, 'interner> {
    pub(crate) fn new(
        elaborator: &'local mut Elaborator<'interner>,
        current_function: Option<FuncId>,
    ) -> Self {
        let in_unconstrained = current_function.is_some_and(|function| {
            elaborator.interner.function_meta(&function).is_unconstrained()
        });

        Self {
            elaborator,
            current_function,
            frame: Frame::default(),
            call_bindings: HashMap::default(),
            in_loop: false,
            evaluation_depth: 0,
            in_unconstrained,
        }
    }

    /// `typ` as seen from the function being interpreted.
    ///
    /// This is the one way the interpreter reads a type from the HIR of the function it is
    /// interpreting: [`Self::expr_type`] and [`Self::bindings`] are shorthands for it.
    pub(super) fn ty(&self, typ: &Type) -> Type {
        // A polymorphic global's HIR keeps its quantifier, and its quantified variables are the
        // ones the use site binds, so substitute underneath it.
        if let Type::Forall(variables, typ) = typ {
            return Type::Forall(variables.clone(), Box::new(self.ty(typ)));
        }
        self.frame.substitute(typ)
    }

    /// The type of the expression `id` as seen from the function being interpreted.
    fn expr_type(&self, id: ExprId) -> Type {
        self.elaborator.interner.try_id_type(id).map_or(Type::Error, |typ| self.ty(typ))
    }

    /// `bindings` with each bound type as seen from the function being interpreted.
    fn bindings(&self, bindings: &TypeBindings) -> TypeBindings {
        bindings
            .iter()
            .map(|(var_id, (var, kind, typ))| (*var_id, (var.clone(), kind.clone(), self.ty(typ))))
            .collect()
    }

    /// `value` with every type it holds as seen from the function being interpreted.
    pub(super) fn value(&self, value: Value) -> Value {
        if self.frame.bindings().is_empty() {
            return value;
        }
        value.map_types(&|typ| self.ty(typ))
    }

    /// `value` with the types solved at runtime since it was built applied: the only bindings of
    /// this frame it can be missing, since everything else was applied when it was built (or by
    /// its caller). This is [`Self::value`], skipped while nothing has been solved at runtime.
    pub(crate) fn apply_runtime_solves(&self, value: Value) -> Value {
        if self.frame.has_runtime_solves() { self.value(value) } else { value }
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
        instantiation_bindings: TypeBindings,
        location: Location,
    ) -> IResult<Value> {
        self.call_function_taking_solves(function, arguments, instantiation_bindings, location)
            .map(|(result, _)| result)
    }

    /// [`Self::call_function`], also returning the type variables the callee solved and handed
    /// back to this frame (see [`Frame::take_solves`]).
    fn call_function_taking_solves(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        instantiation_bindings: TypeBindings,
        location: Location,
    ) -> IResult<(Value, Vec<TypeVariableId>)> {
        self.elaborator.define_function_meta_if_undefined(function);
        let trait_method = self.elaborator.interner.get_trait_item_id(function);

        let mut instantiation_bindings = self.bindings(&instantiation_bindings);
        resolve_type_bindings(&mut instantiation_bindings);

        self.elaborator.push_interpreter_call_stack(location)?;

        let impl_bindings = match compute_impl_bindings(
            self.elaborator.interner,
            trait_method,
            function,
            &instantiation_bindings,
            location,
        ) {
            Ok(impl_bindings) => impl_bindings,
            Err(error) => {
                self.elaborator.pop_interpreter_call_stack();
                return Err(error);
            }
        };

        let mut own_bindings = instantiation_bindings;
        own_bindings.extend(impl_bindings);
        // The caller's frame is restored when the call returns, so the callee extends a copy.
        let callee_frame = self.frame.clone().for_call(&own_bindings);
        let caller_frame = std::mem::replace(&mut self.frame, callee_frame);

        if let Some(tracker) = self.elaborator.evaluation_tracker.as_mut() {
            tracker.track_function_call(function, location);
        }

        // The callee can solve a type that only its own body mentions after building a value that
        // holds it (a `Type` taken by `type_of`, say), so resolve the result before leaving.
        let result = self
            .call_function_inner(function, arguments, location)
            .map(|result| self.apply_runtime_solves(result));

        let callee_frame = std::mem::replace(&mut self.frame, caller_frame);
        let visible: Vec<&Type> = own_bindings.values().map(|(_, _, typ)| typ).collect();
        let solves = self.frame.take_solves(&callee_frame, &visible);

        self.elaborator.pop_interpreter_call_stack();
        result.map(|result| (result, solves))
    }

    /// Helper to check parameter count and dispatch on the function kind to run the function.
    fn call_function_inner(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        location: Location,
    ) -> IResult<Value> {
        let modifiers = self.elaborator.interner.function_modifiers(&function).clone();
        let meta = self.elaborator.function_meta(function);
        if meta.parameters.len() != arguments.len() {
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: meta.parameters.len(),
                actual: arguments.len(),
                location,
            });
        }

        if meta.kind != FunctionKind::Normal {
            let return_type = meta.return_type().clone();
            let return_type = self.ty(&return_type).follow_bindings();
            return self.call_special(function, arguments, return_type, location);
        }

        // Don't change the current function scope if we're in a #[use_callers_scope] function.
        // This will affect where `Expression::resolve`, `Quoted::as_type`, and similar functions resolve.
        let old_function = self.current_function;
        if !modifiers.attributes.has_use_callers_scope() {
            self.current_function = Some(function);
        }

        let previous_in_unconstrained = self.in_unconstrained;
        self.in_unconstrained |= meta.is_unconstrained();

        let result = self.call_user_defined_function(function, arguments, location);

        self.current_function = old_function;
        self.in_unconstrained = previous_in_unconstrained;

        result
    }

    /// Call a non-builtin function
    fn call_user_defined_function(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        location: Location,
    ) -> IResult<Value> {
        let meta = self.elaborator.function_meta(function);
        let parameters = meta.parameters.0.clone();
        let previous_state = self.enter_function();

        for ((parameter, typ, _), (argument, arg_location)) in parameters.iter().zip_eq(arguments) {
            let result = self.define_pattern(parameter, &self.ty(typ), argument, arg_location);
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
        let body_is_unresolved = matches!(
            self.elaborator.function_meta(function).function_body,
            FunctionBody::Unresolved(..)
        );
        match self.elaborator.interner.function(&function).try_as_expr() {
            Some(body) => Ok(body),
            None => {
                if body_is_unresolved {
                    self.elaborator.elaborate_item_from_comptime_in_function(
                        None,
                        None,
                        |elaborator| {
                            elaborator.elaborate_function(function);
                        },
                    );

                    // Recursive call - this will now hit the Some(body) branch
                    self.get_function_body(function, location)
                } else {
                    let function = self.elaborator.interner.function_name(&function).to_owned();
                    Err(InterpreterError::ComptimeDependencyCycle { function, location })
                }
            }
        }
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

        if let Some(name) = func_attrs.builtin() {
            let Some(builtin) = Builtin::lookup(name) else {
                let item = format!("Comptime evaluation for builtin function '{name}'");
                return Err(InterpreterError::Unimplemented { item, location });
            };
            // Builtins read the types stored on their arguments directly, so resolve the types
            // solved since each argument was built. The frame holds every solution its callers
            // can see.
            let arguments = vecmap(arguments, |(argument, location)| {
                (self.apply_runtime_solves(argument), location)
            });
            let result = self.call_builtin(builtin, arguments, return_type, location)?;
            Ok(self.value(result))
        } else if let Some(name) = func_attrs.foreign() {
            let Some(foreign) = Builtin::lookup(name) else {
                let item = format!("Comptime evaluation for foreign function '{name}'");
                return Err(InterpreterError::Unimplemented { item, location });
            };
            self.call_foreign(foreign, arguments, return_type, location)
        } else if let Some(oracle) = func_attrs.oracle() {
            if let Some(ForeignCall::Print) = ForeignCall::lookup(oracle) {
                self.print_oracle(&arguments)
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

    /// Runs `f` with the elaborator resolving in `module`, restoring the module it was resolving
    /// in afterwards (on every exit path, including early returns inside `f`).
    ///
    /// The interpreter's counterpart to [`Elaborator::in_module`], which cannot be used here: `f`
    /// needs `&mut Interpreter`, and the elaborator is borrowed out of it.
    fn in_module<T>(&mut self, module: ModuleId, f: impl FnOnce(&mut Self) -> T) -> T {
        let replaced = self.elaborator.replace_module(module);
        let result = f(self);
        self.elaborator.restore_module(replaced);
        result
    }

    /// Call a closure value with the given arguments and environment, returning the result and
    /// the type variables the closure solved and handed back to this frame (see
    /// [`Frame::take_solves`]).
    fn call_closure(
        &mut self,
        closure: Closure,
        arguments: Vec<(Value, Location)>,
        call_location: Location,
    ) -> IResult<(Value, Vec<TypeVariableId>)> {
        self.elaborator.push_interpreter_call_stack(call_location)?;

        // Resolve the closure body in the scope of the function it was originally evaluated in.
        self.in_module(closure.module_scope, |this| {
            let old_function =
                std::mem::replace(&mut this.current_function, closure.function_scope);

            let callee_frame = this.frame.clone().for_closure(closure.frame);
            let caller_frame = std::mem::replace(&mut this.frame, callee_frame);

            // The body can solve types after a value holding them was built (`[make!()]` types
            // the array before the macro call runs), so resolve the result under the closure's
            // frame before leaving it.
            let result = this
                .call_closure_inner(closure.lambda, closure.env, arguments, call_location)
                .map(|result| this.apply_runtime_solves(result));

            let callee_frame = std::mem::replace(&mut this.frame, caller_frame);
            let solves = this.frame.take_solves(&callee_frame, &[&closure.typ]);
            this.elaborator.pop_interpreter_call_stack();

            this.current_function = old_function;
            result.map(|result| (result, solves))
        })
    }

    /// Performs the bulk of the work for calling a closure function.
    /// This function is very similar to [`Self::call_user_defined_function`]
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

        let parameters = closure.parameters.iter().zip_eq(arguments);
        for ((parameter, typ), (argument, arg_location)) in parameters {
            let result = self.define_pattern(parameter, &self.ty(typ), argument, arg_location);
            if let Err(err) = result {
                self.exit_function(previous_state);
                return Err(err);
            }
        }

        for (param, arg) in closure.captures.into_iter().zip_eq(environment) {
            self.define(param.ident.id, arg);
        }

        let result = self.evaluate(closure.body);

        self.exit_function(previous_state);
        result
    }

    /// Enters a function, pushing a new scope and resetting any required state.
    /// Returns the previous values of the internal state, to be reset when
    /// [`Self::exit_function`] is called.
    ///
    /// The callee sees only its own scopes; see [`ComptimeScopes`](super::ComptimeScopes).
    pub(super) fn enter_function(&mut self) -> (bool, usize) {
        let previous_floor = self.elaborator.comptime_scopes_mut().enter_function();
        (std::mem::take(&mut self.in_loop), previous_floor)
    }

    /// Resets the per-function state to the value previously returned by [`Self::enter_function`]
    pub(super) fn exit_function(&mut self, state: (bool, usize)) {
        self.in_loop = state.0;
        self.elaborator.comptime_scopes_mut().exit_function(state.1);
    }

    /// Pushes a new scope to define any variables in.
    pub(super) fn push_scope(&mut self) {
        self.elaborator.comptime_scopes_mut().push();
    }

    /// Pops the innermost scope.
    pub(super) fn pop_scope(&mut self) {
        self.elaborator.comptime_scopes_mut().pop();
    }

    /// Defines a pattern, putting all variables contained within the pattern in the current scope.
    pub(super) fn define_pattern(
        &mut self,
        pattern: &HirPattern,
        typ: &Type,
        argument: Value,
        location: Location,
    ) -> IResult<()> {
        self.define_pattern_inner(pattern, typ, argument, location, false)
    }

    /// `mutable` is `true` when this pattern is nested inside a `HirPattern::Mutable`,
    /// in which case each leaf identifier is wrapped in a mutable `Value::Pointer`.
    /// The wrapping must happen at each leaf rather than at the composite root: wrapping
    /// the whole tuple/struct value would cause subsequent destructuring to see a
    /// `Value::Pointer` instead of the expected `Value::Tuple`/`Value::Struct`.
    fn define_pattern_inner(
        &mut self,
        pattern: &HirPattern,
        typ: &Type,
        argument: Value,
        location: Location,
        mutable: bool,
    ) -> IResult<()> {
        match pattern {
            HirPattern::Identifier(identifier) => {
                let argument = if mutable {
                    Value::Pointer(ValueCell::new(argument), true, true)
                } else {
                    argument
                };
                self.define(identifier.id, argument);
                Ok(())
            }
            HirPattern::Mutable(pattern, _) => {
                self.define_pattern_inner(pattern, typ, argument, location, true)
            }
            HirPattern::Tuple(pattern_fields, _) => {
                let typ = &typ.follow_bindings();

                match (argument, typ) {
                    (Value::Tuple(fields), Type::Tuple(type_fields))
                        if fields.len() == pattern_fields.len() =>
                    {
                        for ((pattern, typ), argument) in
                            pattern_fields.iter().zip_eq(type_fields).zip_eq(fields)
                        {
                            let argument = argument.borrow().clone();
                            self.define_pattern_inner(pattern, typ, argument, location, mutable)?;
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
                        self.define_pattern_inner(
                            field_pattern,
                            &field_type,
                            field.clone(),
                            location,
                            mutable,
                        )?;
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
        self.elaborator.comptime_scopes_mut().define(id, argument);
    }

    /// Mutate an existing variable, potentially from a prior scope
    fn mutate(&mut self, id: DefinitionId, argument: Value, location: Location) -> IResult<()> {
        // Locals of enclosing callers are not visible, so a callee cannot mutate them.
        let slot = if let Some(local) = self.elaborator.comptime_scopes_mut().get_mut(id) {
            local
        } else if let DefinitionKind::Global(global_id) =
            self.elaborator.interner.definition(id).kind
            && let GlobalValue::Resolved(value) =
                &mut self.elaborator.interner.get_global_mut(global_id).value
        {
            value
        } else {
            return Err(InterpreterError::VariableNotInScope { location });
        };

        match slot {
            Value::Pointer(reference, true, _) => {
                // We can't store to the reference directly, we need to check if the value
                // is a struct or tuple to store to each field instead. This is so any
                // references to these fields are also updated.
                Self::store_flattened(reference, argument);
            }
            _ => *slot = argument,
        }
        Ok(())
    }

    /// Lookup the comptime value of the given variable
    pub(super) fn lookup(&self, ident: &HirIdent) -> IResult<Value> {
        self.lookup_id(ident.id, ident.location)
    }

    /// Lookup the comptime value of the given definition
    pub fn lookup_id(&self, id: DefinitionId, location: Location) -> IResult<Value> {
        // Locals of enclosing callers are not visible, so a callee cannot read them.
        if let Some(value) = self.elaborator.comptime_scopes().get(id) {
            return Ok(value.clone());
        }

        if let DefinitionKind::Global(global_id) = self.elaborator.interner.definition(id).kind
            && let GlobalValue::Resolved(value) =
                &self.elaborator.interner.get_global(global_id).value
        {
            return Ok(value.clone());
        }

        let name = self.elaborator.interner.definition_name(id).to_string();
        Err(InterpreterError::NonComptimeVarReferenced { name, location })
    }

    /// Evaluate an expression and return the result.
    /// This will automatically dereference a mutable variable if used.
    pub fn evaluate(&mut self, id: ExprId) -> IResult<Value> {
        // If comptime evaluation has been halted, don't execute anything
        if self.elaborator.comptime_evaluation_halted() {
            return Err(InterpreterError::SkippedDueToEarlierErrors);
        }

        // Skip expressions that had errors during elaboration and halt all future execution
        if self.elaborator.interner.exprs_with_errors.contains(&id) {
            self.elaborator.halt_comptime_evaluation();
            return Err(InterpreterError::SkippedDueToEarlierErrors);
        }

        match self.evaluate_no_dereference(id)? {
            // An auto-deref pointer (the second flag) stands in for a variable that is
            // dereferenced automatically on use, regardless of whether it is mutable: indexing
            // an immutable array, for instance, yields an immutable auto-deref pointer.
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

        let expr = self.elaborator.interner.expression(&id);
        if let Some(tracker) = self.elaborator.evaluation_tracker.as_mut() {
            tracker.track_expression(&expr, self.elaborator.interner.expr_location(&id));
        }
        let result = match expr {
            HirExpression::Ident(ident, _) => self.evaluate_ident(ident, id),
            HirExpression::Literal(literal) => self.evaluate_literal(literal, id),
            HirExpression::Block(block) => self.evaluate_block(block),
            HirExpression::Prefix(prefix) => self.evaluate_prefix(prefix, id),
            HirExpression::Infix(infix) => self.evaluate_infix(infix, id),
            HirExpression::Index(index) => self.evaluate_index(&index, id),
            HirExpression::Constructor(constructor) => self.evaluate_constructor(constructor, id),
            HirExpression::MemberAccess(access) => self.evaluate_access(&access, id),
            HirExpression::Call(call) => self.evaluate_call(call, id),
            HirExpression::Constrain(constrain) => self.evaluate_constrain(&constrain),
            HirExpression::Cast(cast) => self.evaluate_cast(&cast, id),
            HirExpression::If(if_) => self.evaluate_if(&if_),
            HirExpression::Match(_) => {
                let location = self.elaborator.interner.expr_location(&id);
                Err(InterpreterError::Unimplemented {
                    item: "Match expressions in comptime code".to_string(),
                    location,
                })
            }
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
                self.elaborator.halt_comptime_evaluation();
                Err(InterpreterError::SkippedDueToEarlierErrors)
            }
        };
        self.evaluation_depth -= 1;
        result
    }

    /// Evaluates a variable
    pub(super) fn evaluate_ident(&mut self, ident: HirIdent, id: ExprId) -> IResult<Value> {
        let definition = self.elaborator.interner.definition(ident.id);

        if let ImplKind::TraitItem(item) = &ident.impl_kind {
            return self.evaluate_trait_item(item.id(), id);
        }

        match &definition.kind {
            DefinitionKind::Function(function_id) => {
                let typ = self.expr_type(id).follow_bindings();
                let bindings = self.elaborator.interner.try_get_instantiation_bindings(id);
                let mut bindings = bindings.map_or(TypeBindings::default(), |b| self.bindings(b));
                resolve_type_bindings(&mut bindings);
                Ok(Value::Function(*function_id, typ, Rc::new(bindings)))
            }
            DefinitionKind::Local(_) => self.lookup(&ident),
            DefinitionKind::Global(global_id) => {
                // Avoid resetting the value if it is already known
                let global_id = *global_id;
                let global_info = self.elaborator.interner.get_global(global_id);
                match &global_info.value {
                    GlobalValue::Resolved(value) => {
                        // Track the number of times a global was accessed during execution.
                        // Globals are initialized during compilation; to track their initialization we have to add a tracker
                        // before we try to interpret a specific call already. During interpretation the body is not revisited.
                        if let Some(tracker) = self.elaborator.evaluation_tracker.as_mut() {
                            tracker.track_location(global_info.location);
                        }

                        // Enum variant globals with generics are instantiated with a Type::Forall
                        // We need to resolve the type, but it has already been done by the elaborator
                        if let Value::Enum(tag, fields, _) = value {
                            let typ = self.expr_type(id).follow_bindings();
                            Ok(Value::Enum(*tag, fields.clone(), typ))
                        } else {
                            Ok(value.clone())
                        }
                    }
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
                            // Roll the sentinel back so that a later reference to this same
                            // global isn't misreported as a dependency cycle.
                            self.elaborator.interner.get_global_mut(global_id).value =
                                GlobalValue::Unresolved;
                            let location = self.elaborator.interner.expr_location(&id);
                            Err(InterpreterError::GlobalCouldNotBeResolved { location })
                        }
                    }
                }
            }
            DefinitionKind::NumericGeneric(type_variable, numeric_typ) => {
                let value = self.ty(&Type::TypeVariable(type_variable.clone()));
                self.evaluate_numeric_generic(&value, numeric_typ, id)
            }
            DefinitionKind::AssociatedConstant(trait_impl_id, name) => {
                let typ =
                    self.elaborator.interner.find_associated_type_for_impl(*trait_impl_id, name);
                let typ = typ.expect("Expected to find associated type");
                // The value can mention the impl's generics (`A + B` in
                // `impl<let A: u32, let B: u32>`), which the frame's substitution binds.
                let typ = self.ty(typ);
                let location = self.elaborator.interner.expr_location(&id);
                match typ.evaluate_to_integer(&typ.kind(), location) {
                    Ok(value) => self.evaluate_integer_literal(value.to_bigint(), id),
                    Err(err) => Err(InterpreterError::InvalidAssociatedConstant {
                        err: Box::new(err),
                        location,
                    }),
                }
            }
        }
    }

    /// Evaluates a numeric generic with the value `value` (expected to be `Type::Constant`)
    /// and an expected integer type `expected`.
    fn evaluate_numeric_generic(
        &self,
        value: &Type,
        expected: &Type,
        id: ExprId,
    ) -> IResult<Value> {
        let location = self.elaborator.interner.id_location(id);
        let value = value
            .evaluate_to_integer(&Kind::Numeric(Box::new(expected.clone())), location)
            .map_err(|err| {
                let err = Box::new(err);
                let location = self.elaborator.interner.expr_location(&id);
                InterpreterError::InvalidNumericGeneric { err, location }
            })?;

        self.evaluate_integer_literal(value.to_bigint(), id)
    }

    /// Lazily resolves the trait's method metas (so that downstream helpers like
    /// `bind_trait_impl_func_generics_to_trait_func_generics` can read them),
    /// then delegates to `resolve_trait_item` from the monomorphization module.
    ///
    /// Returns the resolved item and the instantiation bindings of `id` extended with the impl's.
    fn resolve_trait_item(
        &mut self,
        item: TraitItemId,
        id: ExprId,
    ) -> Result<(crate::monomorphization::TraitItem, TypeBindings), InterpreterError> {
        self.elaborator.resolve_trait_method_metas_for(item.trait_id);
        let resolved =
            resolve_trait_item(self.elaborator.interner, item, id, self.frame.bindings())?;
        // The interpreter runs during elaboration, where solving a trait constraint is supposed
        // to commit the inference variables it resolved — the same thing `check_trait_constraints`
        // does for a constraint solved by the type checker.
        Type::apply_type_bindings(resolved.impl_search_bindings);
        Ok((resolved.item, self.bindings(&resolved.instantiation_bindings)))
    }

    fn evaluate_trait_item(&mut self, item: TraitItemId, id: ExprId) -> IResult<Value> {
        let typ = self.expr_type(id).follow_bindings();

        match self.resolve_trait_item(item, id)? {
            (crate::monomorphization::TraitItem::Method(func_id), bindings) => {
                Ok(Value::Function(func_id, typ, Rc::new(bindings)))
            }
            (crate::monomorphization::TraitItem::Constant { id: _, expected_type, value }, _) => {
                // The value can mention the generics of the function being interpreted, e.g.
                // `A + B` for `Self::N` inside a method of `impl<let A: u32, let B: u32>`.
                let value = self.ty(&value);
                self.evaluate_numeric_generic(&value, &expected_type, id)
            }
        }
    }

    fn evaluate_literal(&mut self, literal: HirLiteral, id: ExprId) -> IResult<Value> {
        match literal {
            HirLiteral::Unit => Ok(Value::Unit),
            HirLiteral::Bool(value) => Ok(Value::Bool(value)),
            HirLiteral::Integer(value) => self.evaluate_integer_literal(value, id),
            HirLiteral::Str(string) => Ok(Value::String(Rc::new(string))),
            HirLiteral::FmtStr(fragments, captures, length) => {
                self.evaluate_format_string(fragments, captures, length, id)
            }
            HirLiteral::Array(array) => self.evaluate_array(array, id),
            HirLiteral::Vector(array) => self.evaluate_vector(array, id),
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
        length: u32,
        id: ExprId,
    ) -> IResult<Value> {
        let mut new_fragments = Vec::with_capacity(fragments.len());

        let mut values: VecDeque<_> =
            captures.into_iter().map(|capture| self.evaluate(capture)).collect::<Result<_, _>>()?;

        for fragment in fragments {
            match fragment {
                FmtStrFragment::String(string) => {
                    new_fragments.push(FormatStringFragment::String(string));
                }
                FmtStrFragment::Interpolation(name, _location) => {
                    if let Some(value) = values.pop_front() {
                        new_fragments.push(FormatStringFragment::Value { name, value });
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

        let typ = self.expr_type(id).follow_bindings();
        Ok(Value::FormatString(Rc::new(new_fragments), typ, length))
    }

    /// Since integers are polymorphic, evaluating one requires the result type.
    /// We pass down the result type the elaborator previously inferred.
    fn evaluate_integer_literal(&self, value: BigInt, id: ExprId) -> IResult<Value> {
        let typ = self.expr_type(id).follow_bindings();
        let location = self.elaborator.interner.expr_location(&id);
        Integer::try_from_bigint(&value, &typ).map(Value::Integer).ok_or_else(|| {
            let typ = typ.clone();
            InterpreterError::IntegerOutOfRangeForType { value, typ, location }
        })
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
        let typ = self.expr_type(id).follow_bindings();

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
                match self.ty(&length).evaluate_to_u32(location) {
                    Ok(length) => {
                        let elements = (0..length).map(|_| element.clone()).collect();
                        Ok(Value::Array(elements, typ))
                    }
                    Err(err) => {
                        let err = Box::new(err);
                        let location = self.elaborator.interner.expr_location(&id);
                        Err(InterpreterError::InvalidArrayLength { err, location })
                    }
                }
            }
        }
    }

    fn evaluate_vector(&mut self, array: HirArrayLiteral, id: ExprId) -> IResult<Value> {
        self.evaluate_array(array, id).map(|value| match value {
            Value::Array(array, typ) => Value::Vector(array, typ),
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
            self.evaluate_overloaded_prefix(&prefix, rhs, id)
        } else {
            let location = self.elaborator.interner.expr_location(&id);
            evaluate_prefix_with_value(rhs, prefix.operator, location)
        }
    }

    fn evaluate_infix(&mut self, infix: HirInfixExpression, id: ExprId) -> IResult<Value> {
        let lhs_value = self.evaluate(infix.lhs)?;
        let rhs_value = self.evaluate(infix.rhs)?;

        if self.elaborator.interner.get_selected_impl_for_expression(id).is_some() {
            return self.evaluate_overloaded_infix(&infix, lhs_value, rhs_value, id);
        }

        let location = self.elaborator.interner.expr_location(&id);

        infix::evaluate_infix(lhs_value, rhs_value, infix.operator, location)
    }

    fn evaluate_overloaded_infix(
        &mut self,
        infix: &HirInfixExpression,
        lhs: Value,
        rhs: Value,
        id: ExprId,
    ) -> IResult<Value> {
        let method = infix
            .trait_method_id
            .unwrap_or_else(|| panic!("Interpreter::evaluate_overloaded_infix: expected operator method to be resolved for {:?}", infix.operator));
        let operator = infix.operator.kind;

        let (method, type_bindings) = self.resolve_trait_item(method, id)?;
        let method_id = method.unwrap_method();

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
                self.evaluate_ordering(&value, operator, location)
            }
            _ => Ok(value),
        }
    }

    fn evaluate_overloaded_prefix(
        &mut self,
        prefix: &HirPrefixExpression,
        rhs: Value,
        id: ExprId,
    ) -> IResult<Value> {
        let method =
            prefix.trait_method_id.expect("ice: expected prefix operator trait at this point");

        let (method, type_bindings) = self.resolve_trait_item(method, id)?;
        let method_id = method.unwrap_method();

        let rhs = (rhs, self.elaborator.interner.expr_location(&prefix.rhs));

        let location = self.elaborator.interner.expr_location(&id);
        self.call_function(method_id, vec![rhs], type_bindings, location)
    }

    /// Given the result of a `cmp` operation, convert it into the boolean result of the given operator.
    fn evaluate_ordering(
        &self,
        ordering: &Value,
        operator: BinaryOpKind,
        location: Location,
    ) -> IResult<Value> {
        let field_ordering = match ordering {
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
                            Value::Integer(Integer::Field(ordering)) => Some(*ordering),
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
            BinaryOpKind::LessEqual => ordering != 2_u128.into(),
            // `>`:  `ordering == Ordering::Greater`
            BinaryOpKind::Greater => ordering == 2_u128.into(),
            // `>=`: `ordering != Ordering::Less`
            BinaryOpKind::GreaterEqual => !ordering.is_zero(),
            _ => unreachable!("evaluate_ordering called with non-ordering operator"),
        };
        Ok(Value::Bool(result))
    }

    fn evaluate_index(&mut self, index: &HirIndexExpression, id: ExprId) -> IResult<Value> {
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
                let field_value = ValueCell::new(self.evaluate(expr)?);
                Ok((Rc::new(name.into_string()), field_value))
            })
            .collect::<Result<_, _>>()?;

        let typ = self.expr_type(id).follow_bindings();
        Ok(Value::Struct(fields, typ))
    }

    /// Unlike a struct constructor, an enum constructor inserts a tag value along with the fields
    fn evaluate_enum_constructor(
        &mut self,
        constructor: HirEnumConstructorExpression,
        id: ExprId,
    ) -> IResult<Value> {
        let fields = try_vecmap(constructor.arguments, |arg| self.evaluate(arg))?;
        let typ = self
            .elaborator
            .interner
            .try_id_type(id)
            .map_or(Type::Error, |typ| self.ty(typ.unwrap_forall().1))
            .follow_bindings();
        Ok(Value::Enum(constructor.variant_index, fields, typ))
    }

    fn evaluate_access(&mut self, access: &HirMemberAccess, id: ExprId) -> IResult<Value> {
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
    fn get_field(&mut self, value: Value, id: ExprId, name: &String) -> IResult<ValueCell> {
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
        let field_name = name.clone();
        Err(InterpreterError::ExpectedStructToHaveField { typ, field_name, location })
    }

    /// Evaluates a call expression, deferring to [`Self::call_function`] or [`Self::call_closure`]
    /// once the function is determined.
    fn evaluate_call(&mut self, call: HirCallExpression, id: ExprId) -> IResult<Value> {
        if let Some(var_ids) = self.call_bindings.remove(&id) {
            self.frame.forget(&var_ids);
        }

        let function = self.evaluate(call.func)?;
        let arguments = try_vecmap(call.arguments, |arg| {
            Ok((self.evaluate(arg)?, self.elaborator.interner.expr_location(&arg)))
        })?;
        let location = self.elaborator.interner.expr_location(&id);

        let (result, solves) = match function {
            Value::Function(function_id, _, bindings) => {
                let bindings = unwrap_rc(bindings);
                let (mut result, mut solves) =
                    self.call_function_taking_solves(function_id, arguments, bindings, location)?;
                if call.is_macro_call {
                    let expr = result.into_expression(self.elaborator, location)?;
                    let expr = self.elaborator.elaborate_item_from_comptime_in_function(
                        self.current_function,
                        None,
                        |elaborator| elaborator.elaborate_expression(expr).0,
                    );
                    result = self.evaluate(expr)?;
                    solves.extend(
                        self.unify_macro_call_result_with_expected_type(id, location, &result),
                    );
                } else {
                    solves.extend(self.solve_call_type_from_result(id, &result));
                }
                (result, solves)
            }
            Value::Closure(closure) => {
                let (result, mut solves) = self.call_closure(*closure, arguments, location)?;
                solves.extend(self.solve_call_type_from_result(id, &result));
                (result, solves)
            }
            value => {
                let typ = value.get_type().into_owned();
                return Err(InterpreterError::NonFunctionCalled { typ, location });
            }
        };

        self.call_bindings.insert(id, solves);
        Ok(result)
    }

    /// Macro calls are typed as type variables during type checking. Once the call has produced
    /// a value, unify its type with the expression's and add what that solves to the frame, so
    /// that the rest of the function sees the macro call's type. Returns the type variables
    /// solved.
    fn unify_macro_call_result_with_expected_type(
        &mut self,
        id: ExprId,
        location: Location,
        result: &Value,
    ) -> Vec<TypeVariableId> {
        let expected_type = self.expr_type(id);
        let actual_type = result.get_type();

        let mut bindings = TypeBindings::default();
        match actual_type.try_unify(&expected_type, &mut bindings) {
            Ok(()) => {
                let solved = bindings.keys().copied().collect();
                self.frame.solve(bindings);
                solved
            }
            Err(UnificationError) => {
                self.elaborator.push_err(self.elaborator.new_type_mismatch_error(
                    &actual_type,
                    &expected_type,
                    location,
                ));
                Vec::new()
            }
        }
    }

    /// A call can return the value of a macro call that ran in another frame: the body of a
    /// closure, or a function the closure was passed to. If the call's type is still unsolved in
    /// this frame, solve it from the value the call produced. Returns the type variables solved.
    fn solve_call_type_from_result(&mut self, id: ExprId, result: &Value) -> Vec<TypeVariableId> {
        let expected_type = self.expr_type(id);
        if !expected_type.contains_unbound_type_variable() {
            return Vec::new();
        }

        let mut bindings = TypeBindings::default();
        if result.get_type().try_unify(&expected_type, &mut bindings).is_err() {
            return Vec::new();
        }
        let solved = bindings.keys().copied().collect();
        self.frame.solve(bindings);
        solved
    }

    fn evaluate_cast(&mut self, cast: &HirCastExpression, id: ExprId) -> IResult<Value> {
        let evaluated_lhs = self.evaluate(cast.lhs)?;
        let location = self.elaborator.interner.expr_location(&id);
        evaluate_cast_one_step(&self.ty(&cast.r#type), location, evaluated_lhs)
    }

    fn evaluate_if(&mut self, if_: &HirIfExpression) -> IResult<Value> {
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
        let fields = try_vecmap(tuple, |field| Ok(ValueCell::new(self.evaluate(field)?)))?;
        Ok(Value::Tuple(fields))
    }

    fn evaluate_lambda(&self, lambda: HirLambda, id: ExprId) -> IResult<Value> {
        let location = self.elaborator.interner.expr_location(&id);
        let env = try_vecmap(&lambda.captures, |capture| {
            let value = self.lookup_id(capture.ident.id, location)?;
            let value = match value {
                // Dereference mutable variables to capture by value
                Value::Pointer(elem, true, _) => Ok(elem.unwrap_or_clone()),
                other => Ok(other),
            }?;
            Ok(value.move_struct())
        })?;

        let typ = self.expr_type(id).follow_bindings();
        let module_scope = self.elaborator.module_id();
        let closure = Closure {
            lambda,
            env,
            typ,
            function_scope: self.current_function,
            module_scope,
            frame: self.frame.clone(),
        };
        Ok(Value::Closure(Box::new(closure)))
    }

    fn evaluate_quote(&mut self, tokens: Tokens) -> IResult<Value> {
        let tokens = self.substitute_unquoted_values_into_tokens(tokens)?;
        Ok(Value::Quoted(Rc::new(tokens)))
    }

    pub fn evaluate_statement(&mut self, statement: StmtId) -> IResult<Value> {
        // If comptime evaluation has been halted, don't execute anything
        if self.elaborator.comptime_evaluation_halted() {
            return Err(InterpreterError::SkippedDueToEarlierErrors);
        }

        // Skip statements that had errors during elaboration and halt all future execution
        if self.elaborator.interner.stmts_with_errors.contains(&statement) {
            self.elaborator.halt_comptime_evaluation();
            return Err(InterpreterError::SkippedDueToEarlierErrors);
        }

        match self.elaborator.interner.statement(&statement) {
            HirStatement::Let(let_) => self.evaluate_let(let_),
            HirStatement::Assign(assign) => self.evaluate_assign(assign),
            HirStatement::For(for_) => self.evaluate_for(&for_),
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
                self.elaborator.halt_comptime_evaluation();
                Err(InterpreterError::SkippedDueToEarlierErrors)
            }
            HirStatement::TraitAssociatedConstant => {
                let location = self.elaborator.interner.id_location(statement);
                Err(InterpreterError::ErrorNodeEncountered { location })
            }
        }
    }

    pub(crate) fn evaluate_let(&mut self, let_: HirLetStatement) -> IResult<Value> {
        let rhs = self.evaluate(let_.expression)?;
        let location = self.elaborator.interner.expr_location(&let_.expression);
        self.define_pattern(&let_.pattern, &self.ty(&let_.r#type), rhs, location)?;
        Ok(Value::Unit)
    }

    fn evaluate_constrain(&mut self, constrain: &HirConstrainExpression) -> IResult<Value> {
        match self.evaluate(constrain.0)? {
            Value::Bool(true) => Ok(Value::Unit),
            Value::Bool(false) => {
                let location = self.elaborator.interner.expr_location(&constrain.0);
                let message = constrain.2.and_then(|expr| self.evaluate(expr).ok());
                let message = message.map(|value| {
                    value.display(self.elaborator.interner, self.elaborator.files).to_string()
                });
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
                        // The pointee can outlive this frame (`&mut` parameters point into the
                        // caller), and types this frame has solved since `rhs` was built live
                        // only in its substitution, so resolve them before storing.
                        Self::store_flattened(&value, self.apply_runtime_solves(rhs));
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
                        fields[index] = ValueCell::new(rhs);
                        self.store_lvalue(*object, Value::Tuple(fields))
                    }
                    Value::Struct(mut fields, typ) => {
                        fields.insert(Rc::new(field_name.into_string()), ValueCell::new(rhs));
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
                    _ => Value::Vector,
                };

                let typ = array_value.get_type().into_owned();
                let (elements, index) = bounds_check(array_value, index, location)?;

                let new_array = constructor(elements.update(index, rhs), typ);
                self.store_lvalue(*array, new_array)
            }
            HirLValue::Error { location } => Err(InterpreterError::VariableNotInScope { location }),
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
    fn store_flattened(lvalue: &ValueCell, rvalue: Value) {
        let lvalue_ref = lvalue.borrow();
        match (&*lvalue_ref, rvalue) {
            (Value::Struct(lvalue_fields, _), Value::Struct(mut rvalue_fields, _)) => {
                for (name, lvalue) in lvalue_fields {
                    let Some(rvalue) = rvalue_fields.remove(name) else {
                        // Defensive check: If we reach here, it indicates a type system bug
                        panic!(
                            "ICE: store_flattened encountered a struct field mismatch. \
                            Struct field '{name}' exists in lvalue but not in rvalue. \
                            This should have been caught by type checking.",
                        );
                    };
                    Self::store_flattened(lvalue, rvalue.unwrap_or_clone());
                }
            }
            (Value::Tuple(lvalue_fields), Value::Tuple(rvalue_fields)) => {
                // Defensive check: tuple lengths should match. If they do not, it indicates a type system bug
                assert_eq!(
                    lvalue_fields.len(),
                    rvalue_fields.len(),
                    "ICE: store_flattened encountered a tuple length mismatch. \
                    This should have been caught by type checking."
                );

                for (lvalue, rvalue) in lvalue_fields.iter().zip_eq(rvalue_fields) {
                    Self::store_flattened(lvalue, rvalue.unwrap_or_clone());
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
            HirLValue::Error { location } => {
                Err(InterpreterError::VariableNotInScope { location: *location })
            }
        }
    }

    fn evaluate_for(&mut self, for_: &HirForStatement) -> IResult<Value> {
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
                Value::Integer(Integer::I8(_)) => |i| Value::Integer(Integer::I8(i as i8)),
                Value::Integer(Integer::I16(_)) => |i| Value::Integer(Integer::I16(i as i16)),
                Value::Integer(Integer::I32(_)) => |i| Value::Integer(Integer::I32(i as i32)),
                Value::Integer(Integer::I64(_)) => |i| Value::Integer(Integer::I64(i as i64)),
                _ => unreachable!("Checked above that value is signed type"),
            };

            // i128 can store all values from i8 - u64
            let start = to_i128(&start_value).expect("Checked above that value is signed type");
            let end = to_i128(&end_value).expect("Checked above that types match");

            if for_.inclusive {
                self.evaluate_for_loop(start..=end, get_index, for_.identifier.id, for_.block)
            } else {
                self.evaluate_for_loop(start..end, get_index, for_.identifier.id, for_.block)
            }
        } else if start_type.is_unsigned() {
            let get_index = match start_value {
                Value::Integer(Integer::U8(_)) => |i| Value::Integer(Integer::U8(i as u8)),
                Value::Integer(Integer::U16(_)) => |i| Value::Integer(Integer::U16(i as u16)),
                Value::Integer(Integer::U32(_)) => |i| Value::Integer(Integer::U32(i as u32)),
                Value::Integer(Integer::U64(_)) => |i| Value::Integer(Integer::U64(i as u64)),
                Value::Integer(Integer::U128(_)) => |i| Value::Integer(Integer::U128(i)),
                _ => unreachable!("Checked above that value is unsigned type"),
            };

            // u128 can store all values from u8 - u128
            let start = to_u128(&start_value).expect("Checked above that value is unsigned type");
            let end = to_u128(&end_value).expect("Checked above that types match");

            if for_.inclusive {
                self.evaluate_for_loop(start..=end, get_index, for_.identifier.id, for_.block)
            } else {
                self.evaluate_for_loop(start..end, get_index, for_.identifier.id, for_.block)
            }
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
            self.define(index_id, get_index(i));

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

    fn evaluate_break(&self, id: StmtId) -> IResult<Value> {
        if self.in_loop {
            Err(InterpreterError::Break)
        } else {
            let location = self.elaborator.interner.statement_location(id);
            Err(InterpreterError::BreakNotInLoop { location })
        }
    }

    fn evaluate_continue(&self, id: StmtId) -> IResult<Value> {
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

    fn print_oracle(&self, arguments: &[(Value, Location)]) -> Result<Value, InterpreterError> {
        assert_eq!(arguments.len(), 2);

        let Some(output) = self.elaborator.interpreter_output else {
            return Ok(Value::Unit);
        };

        let mut output = output.borrow_mut();

        let print_newline = arguments[0].0 == Value::Bool(true);
        let contents = arguments[1].0.display(self.elaborator.interner, self.elaborator.files);
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

/// Bounds check the given array and index pair.
/// This will also ensure the given arguments are in fact an array and u32.
fn bounds_check(array: Value, index: Value, location: Location) -> IResult<(Vector<Value>, usize)> {
    let collection = match array {
        Value::Array(array, _) => array,
        Value::Vector(array, _) => array,
        value => {
            let typ = value.get_type().into_owned();
            return Err(InterpreterError::NonArrayIndexed { typ, location });
        }
    };

    let index = match index {
        Value::Integer(Integer::U32(value)) => value as usize,
        value => {
            let typ = value.get_type().into_owned();
            let expected_type = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);
            return Err(InterpreterError::TypeMismatch {
                expected: expected_type.to_string(),
                actual: typ,
                location,
            });
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
            Value::Integer(Integer::Field(value)) => Ok(Value::field(-value)),
            Value::Integer(Integer::I8(value)) => value
                .checked_neg()
                .map(Value::i8)
                .ok_or_else(|| InterpreterError::NegateWithOverflow { location }),
            Value::Integer(Integer::I16(value)) => value
                .checked_neg()
                .map(Value::i16)
                .ok_or_else(|| InterpreterError::NegateWithOverflow { location }),
            Value::Integer(Integer::I32(value)) => value
                .checked_neg()
                .map(Value::i32)
                .ok_or_else(|| InterpreterError::NegateWithOverflow { location }),
            Value::Integer(Integer::I64(value)) => value
                .checked_neg()
                .map(Value::i64)
                .ok_or_else(|| InterpreterError::NegateWithOverflow { location }),
            Value::Integer(Integer::U8(_)) => {
                Err(InterpreterError::CannotApplyMinusToType { location, typ: "u8" })
            }
            Value::Integer(Integer::U16(_)) => {
                Err(InterpreterError::CannotApplyMinusToType { location, typ: "u16" })
            }
            Value::Integer(Integer::U32(_)) => {
                Err(InterpreterError::CannotApplyMinusToType { location, typ: "u32" })
            }
            Value::Integer(Integer::U64(_)) => {
                Err(InterpreterError::CannotApplyMinusToType { location, typ: "u64" })
            }
            Value::Integer(Integer::U128(_)) => {
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
            Value::Integer(Integer::I8(value)) => Ok(Value::i8(!value)),
            Value::Integer(Integer::I16(value)) => Ok(Value::i16(!value)),
            Value::Integer(Integer::I32(value)) => Ok(Value::i32(!value)),
            Value::Integer(Integer::I64(value)) => Ok(Value::i64(!value)),
            Value::Integer(Integer::U8(value)) => Ok(Value::u8(!value)),
            Value::Integer(Integer::U16(value)) => Ok(Value::u16(!value)),
            Value::Integer(Integer::U32(value)) => Ok(Value::u32(!value)),
            Value::Integer(Integer::U64(value)) => Ok(Value::u64(!value)),
            Value::Integer(Integer::U128(value)) => Ok(Value::u128(!value)),
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
                other => Ok(Value::Pointer(ValueCell::new(other), false, mutable)),
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

fn to_u128(value: &Value) -> Option<u128> {
    match value {
        Value::Integer(Integer::U8(value)) => Some(u128::from(*value)),
        Value::Integer(Integer::U16(value)) => Some(u128::from(*value)),
        Value::Integer(Integer::U32(value)) => Some(u128::from(*value)),
        Value::Integer(Integer::U64(value)) => Some(u128::from(*value)),
        Value::Integer(Integer::U128(value)) => Some(*value),
        _ => None,
    }
}

fn to_i128(value: &Value) -> Option<i128> {
    match value {
        Value::Integer(Integer::I8(value)) => Some(i128::from(*value)),
        Value::Integer(Integer::I16(value)) => Some(i128::from(*value)),
        Value::Integer(Integer::I32(value)) => Some(i128::from(*value)),
        Value::Integer(Integer::I64(value)) => Some(i128::from(*value)),
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
        let enabled_unstable_features =
            &self.required_unstable_features.get(&crate_id).cloned().unwrap_or_default();
        let cli_options = ElaboratorOptions {
            debug_comptime_in_file: None,

            enabled_unstable_features,
            disable_required_unstable_features: false,
        };
        let module_id = ModuleId { krate: crate_id, local_id };

        let mut elaborator = Elaborator::from_context(self, crate_id, cli_options);
        elaborator.setup_interpreter_for(module_id, |interpreter| {
            let instantiation_bindings = TypeBindings::default();
            interpreter.call_function(main_id, args, instantiation_bindings, location)
        })
    }
}
