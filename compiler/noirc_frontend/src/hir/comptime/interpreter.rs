use std::collections::VecDeque;
use std::{collections::hash_map::Entry, rc::Rc};

use acvm::{acir::AcirField, FieldElement};
use im::Vector;
use iter_extended::try_vecmap;
use noirc_errors::Location;
use rustc_hash::FxHashMap as HashMap;

use crate::ast::{BinaryOpKind, FunctionKind, IntegerBitSize, Signedness};
use crate::elaborator::Elaborator;
use crate::graph::CrateId;
use crate::hir_def::expr::ImplKind;
use crate::hir_def::function::FunctionBody;
use crate::macros_api::UnaryOp;
use crate::monomorphization::{
    perform_impl_bindings, perform_instantiation_bindings, resolve_trait_method,
    undo_instantiation_bindings,
};
use crate::token::Tokens;
use crate::TypeVariable;
use crate::{
    hir_def::{
        expr::{
            HirArrayLiteral, HirBlockExpression, HirCallExpression, HirCastExpression,
            HirConstructorExpression, HirExpression, HirIdent, HirIfExpression, HirIndexExpression,
            HirInfixExpression, HirLambda, HirMemberAccess, HirMethodCallExpression,
            HirPrefixExpression,
        },
        stmt::{
            HirAssignStatement, HirConstrainStatement, HirForStatement, HirLValue, HirLetStatement,
            HirPattern,
        },
    },
    macros_api::{HirLiteral, HirStatement, NodeInterner},
    node_interner::{DefinitionId, DefinitionKind, ExprId, FuncId, StmtId},
    Shared, Type, TypeBinding, TypeBindings, TypeVariableKind,
};

use super::errors::{IResult, InterpreterError};
use super::value::{unwrap_rc, Value};

mod builtin;
mod foreign;
mod unquote;

#[allow(unused)]
pub struct Interpreter<'local, 'interner> {
    /// To expand macros the Interpreter needs access to the Elaborator
    pub elaborator: &'local mut Elaborator<'interner>,

    crate_id: CrateId,

    in_loop: bool,

    current_function: Option<FuncId>,

    /// Maps each bound generic to each binding it has in the current callstack.
    /// Since the interpreter monomorphizes as it interprets, we can bind over the same generic
    /// multiple times. Without this map, when one of these inner functions exits we would
    /// unbind the generic completely instead of resetting it to its previous binding.
    bound_generics: Vec<HashMap<TypeVariable, Type>>,
}

#[allow(unused)]
impl<'local, 'interner> Interpreter<'local, 'interner> {
    pub(crate) fn new(
        elaborator: &'local mut Elaborator<'interner>,
        crate_id: CrateId,
        current_function: Option<FuncId>,
    ) -> Self {
        let bound_generics = Vec::new();
        Self { elaborator, crate_id, current_function, bound_generics, in_loop: false }
    }

    pub(crate) fn call_function(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        mut instantiation_bindings: TypeBindings,
        location: Location,
    ) -> IResult<Value> {
        let trait_method = self.elaborator.interner.get_trait_method_id(function);

        // To match the monomorphizer, we need to call follow_bindings on each of
        // the instantiation bindings before we unbind the generics from the previous function.
        // This is because the instantiation bindings refer to variables from the call site.
        for (_, binding) in instantiation_bindings.values_mut() {
            *binding = binding.follow_bindings();
        }

        self.unbind_generics_from_previous_function();
        perform_instantiation_bindings(&instantiation_bindings);
        let mut impl_bindings =
            perform_impl_bindings(self.elaborator.interner, trait_method, function, location)?;

        for (_, binding) in impl_bindings.values_mut() {
            *binding = binding.follow_bindings();
        }

        self.remember_bindings(&instantiation_bindings, &impl_bindings);
        let result = self.call_function_inner(function, arguments, location);

        undo_instantiation_bindings(impl_bindings);
        undo_instantiation_bindings(instantiation_bindings);
        self.rebind_generics_from_previous_function();
        result
    }

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

        // Wait until after call_special to set the current function so that builtin functions like
        // `.as_type()` still call the resolver in the caller's scope.
        let old_function = self.current_function.replace(function);
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
            self.define_pattern(parameter, typ, argument, arg_location)?;
        }

        let function_body = self.get_function_body(function, location)?;
        let result = self.evaluate(function_body)?;
        self.exit_function(previous_state);
        Ok(result)
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
                    self.elaborate_item(None, |elaborator| {
                        elaborator.elaborate_function(function);
                    });

                    self.get_function_body(function, location)
                } else {
                    let function = self.elaborator.interner.function_name(&function).to_owned();
                    Err(InterpreterError::ComptimeDependencyCycle { function, location })
                }
            }
        }
    }

    fn elaborate_item<T>(
        &mut self,
        function: Option<FuncId>,
        f: impl FnOnce(&mut Elaborator) -> T,
    ) -> T {
        self.unbind_generics_from_previous_function();
        let result = self.elaborator.elaborate_item_from_comptime(function, f);
        self.rebind_generics_from_previous_function();
        result
    }

    fn call_special(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        return_type: Type,
        location: Location,
    ) -> IResult<Value> {
        let attributes = self.elaborator.interner.function_attributes(&function);
        let func_attrs = attributes.function.as_ref()
            .expect("all builtin functions must contain a function  attribute which contains the opcode which it links to");

        if let Some(builtin) = func_attrs.builtin() {
            let builtin = builtin.clone();
            self.call_builtin(&builtin, arguments, return_type, location)
        } else if let Some(foreign) = func_attrs.foreign() {
            let foreign = foreign.clone();
            foreign::call_foreign(self.elaborator.interner, &foreign, arguments, location)
        } else if let Some(oracle) = func_attrs.oracle() {
            if oracle == "print" {
                self.print_oracle(arguments)
            // Ignore debugger functions
            } else if oracle.starts_with("__debug") {
                Ok(Value::Unit)
            } else {
                let item = format!("Comptime evaluation for oracle functions like {oracle}");
                Err(InterpreterError::Unimplemented { item, location })
            }
        } else {
            let name = self.elaborator.interner.function_name(&function);
            unreachable!("Non-builtin, lowlevel or oracle builtin fn '{name}'")
        }
    }

    fn call_closure(
        &mut self,
        closure: HirLambda,
        environment: Vec<Value>,
        arguments: Vec<(Value, Location)>,
        call_location: Location,
    ) -> IResult<Value> {
        let previous_state = self.enter_function();

        if closure.parameters.len() != arguments.len() {
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: closure.parameters.len(),
                actual: arguments.len(),
                location: call_location,
            });
        }

        let parameters = closure.parameters.iter().zip(arguments);
        for ((parameter, typ), (argument, arg_location)) in parameters {
            self.define_pattern(parameter, typ, argument, arg_location)?;
        }

        for (param, arg) in closure.captures.into_iter().zip(environment) {
            self.define(param.ident.id, arg);
        }

        let result = self.evaluate(closure.body)?;

        self.exit_function(previous_state);
        Ok(result)
    }

    /// Enters a function, pushing a new scope and resetting any required state.
    /// Returns the previous values of the internal state, to be reset when
    /// `exit_function` is called.
    pub(super) fn enter_function(&mut self) -> (bool, Vec<HashMap<DefinitionId, Value>>) {
        // Drain every scope except the global scope
        let mut scope = Vec::new();
        if self.elaborator.interner.comptime_scopes.len() > 1 {
            scope = self.elaborator.interner.comptime_scopes.drain(1..).collect();
        }
        self.push_scope();
        (std::mem::take(&mut self.in_loop), scope)
    }

    pub(super) fn exit_function(&mut self, mut state: (bool, Vec<HashMap<DefinitionId, Value>>)) {
        self.in_loop = state.0;

        // Keep only the global scope
        self.elaborator.interner.comptime_scopes.truncate(1);
        self.elaborator.interner.comptime_scopes.append(&mut state.1);
    }

    pub(super) fn push_scope(&mut self) {
        self.elaborator.interner.comptime_scopes.push(HashMap::default());
    }

    pub(super) fn pop_scope(&mut self) {
        self.elaborator.interner.comptime_scopes.pop();
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<DefinitionId, Value> {
        // the global scope is always at index zero, so this is always Some
        self.elaborator.interner.comptime_scopes.last_mut().unwrap()
    }

    fn unbind_generics_from_previous_function(&mut self) {
        if let Some(bindings) = self.bound_generics.last() {
            for var in bindings.keys() {
                var.unbind(var.id());
            }
        }
        // Push a new bindings list for the current function
        self.bound_generics.push(HashMap::default());
    }

    fn rebind_generics_from_previous_function(&mut self) {
        // Remove the currently bound generics first.
        self.bound_generics.pop();

        if let Some(bindings) = self.bound_generics.last() {
            for (var, binding) in bindings {
                var.force_bind(binding.clone());
            }
        }
    }

    fn remember_bindings(&mut self, main_bindings: &TypeBindings, impl_bindings: &TypeBindings) {
        let bound_generics = self
            .bound_generics
            .last_mut()
            .expect("remember_bindings called with no bound_generics on the stack");

        for (var, binding) in main_bindings.values() {
            bound_generics.insert(var.clone(), binding.follow_bindings());
        }

        for (var, binding) in impl_bindings.values() {
            bound_generics.insert(var.clone(), binding.follow_bindings());
        }
    }

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
                let argument = Value::Pointer(Shared::new(argument), true);
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
                            self.define_pattern(pattern, typ, argument, location)?;
                        }
                        Ok(())
                    }
                    (value, _) => {
                        let actual = value.get_type().into_owned();
                        Err(InterpreterError::TypeMismatch {
                            expected: typ.clone(),
                            actual,
                            location,
                        })
                    }
                }
            }
            HirPattern::Struct(struct_type, pattern_fields, _) => {
                self.push_scope();

                let res = match argument {
                    Value::Struct(fields, struct_type) if fields.len() == pattern_fields.len() => {
                        for (field_name, field_pattern) in pattern_fields {
                            let field = fields.get(&field_name.0.contents).ok_or_else(|| {
                                InterpreterError::ExpectedStructToHaveField {
                                    typ: struct_type.clone(),
                                    field_name: field_name.0.contents.clone(),
                                    location,
                                }
                            })?;

                            let field_type = field.get_type().into_owned();
                            self.define_pattern(
                                field_pattern,
                                &field_type,
                                field.clone(),
                                location,
                            )?;
                        }
                        Ok(())
                    }
                    value => Err(InterpreterError::TypeMismatch {
                        expected: typ.clone(),
                        actual: value.get_type().into_owned(),
                        location,
                    }),
                };
                self.pop_scope();
                res
            }
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
                entry.insert(argument);
                return Ok(());
            }
        }
        Err(InterpreterError::VariableNotInScope { location })
    }

    pub(super) fn lookup(&self, ident: &HirIdent) -> IResult<Value> {
        self.lookup_id(ident.id, ident.location)
    }

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
        match self.evaluate_no_dereference(id)? {
            Value::Pointer(elem, true) => Ok(elem.borrow().clone()),
            other => Ok(other),
        }
    }

    /// Evaluating a mutable variable will dereference it automatically.
    /// This function should be used when that is not desired - e.g. when
    /// compiling a `&mut var` expression to grab the original reference.
    fn evaluate_no_dereference(&mut self, id: ExprId) -> IResult<Value> {
        match self.elaborator.interner.expression(&id) {
            HirExpression::Ident(ident, _) => self.evaluate_ident(ident, id),
            HirExpression::Literal(literal) => self.evaluate_literal(literal, id),
            HirExpression::Block(block) => self.evaluate_block(block),
            HirExpression::Prefix(prefix) => self.evaluate_prefix(prefix, id),
            HirExpression::Infix(infix) => self.evaluate_infix(infix, id),
            HirExpression::Index(index) => self.evaluate_index(index, id),
            HirExpression::Constructor(constructor) => self.evaluate_constructor(constructor, id),
            HirExpression::MemberAccess(access) => self.evaluate_access(access, id),
            HirExpression::Call(call) => self.evaluate_call(call, id),
            HirExpression::MethodCall(call) => self.evaluate_method_call(call, id),
            HirExpression::Cast(cast) => self.evaluate_cast(&cast, id),
            HirExpression::If(if_) => self.evaluate_if(if_, id),
            HirExpression::Tuple(tuple) => self.evaluate_tuple(tuple),
            HirExpression::Lambda(lambda) => self.evaluate_lambda(lambda, id),
            HirExpression::Quote(tokens) => self.evaluate_quote(tokens, id),
            HirExpression::Comptime(block) => self.evaluate_block(block),
            HirExpression::Unquote(tokens) => {
                // An Unquote expression being found is indicative of a macro being
                // expanded within another comptime fn which we don't currently support.
                let location = self.elaborator.interner.expr_location(&id);
                Err(InterpreterError::UnquoteFoundDuringEvaluation { location })
            }
            HirExpression::Error => {
                let location = self.elaborator.interner.expr_location(&id);
                Err(InterpreterError::ErrorNodeEncountered { location })
            }
        }
    }

    pub(super) fn evaluate_ident(&mut self, ident: HirIdent, id: ExprId) -> IResult<Value> {
        let definition = self.elaborator.interner.try_definition(ident.id).ok_or_else(|| {
            let location = self.elaborator.interner.expr_location(&id);
            InterpreterError::VariableNotInScope { location }
        })?;

        if let ImplKind::TraitMethod(method, _, _) = ident.impl_kind {
            let method_id = resolve_trait_method(self.elaborator.interner, method, id)?;
            let typ = self.elaborator.interner.id_type(id).follow_bindings();
            let bindings = self.elaborator.interner.get_instantiation_bindings(id).clone();
            return Ok(Value::Function(method_id, typ, Rc::new(bindings)));
        }

        match &definition.kind {
            DefinitionKind::Function(function_id) => {
                let typ = self.elaborator.interner.id_type(id).follow_bindings();
                let bindings =
                    Rc::new(self.elaborator.interner.get_instantiation_bindings(id).clone());
                Ok(Value::Function(*function_id, typ, bindings))
            }
            DefinitionKind::Local(_) => self.lookup(&ident),
            DefinitionKind::Global(global_id) => {
                // Avoid resetting the value if it is already known
                if let Ok(value) = self.lookup(&ident) {
                    Ok(value)
                } else {
                    let crate_of_global = self.elaborator.interner.get_global(*global_id).crate_id;
                    let let_ =
                        self.elaborator.interner.get_global_let_statement(*global_id).ok_or_else(
                            || {
                                let location = self.elaborator.interner.expr_location(&id);
                                InterpreterError::VariableNotInScope { location }
                            },
                        )?;

                    if let_.comptime || crate_of_global != self.crate_id {
                        self.evaluate_let(let_.clone())?;
                    }
                    self.lookup(&ident)
                }
            }
            DefinitionKind::GenericType(type_variable) => {
                let value = match &*type_variable.borrow() {
                    TypeBinding::Unbound(_) => None,
                    TypeBinding::Bound(binding) => binding.evaluate_to_u32(),
                };

                if let Some(value) = value {
                    let typ = self.elaborator.interner.id_type(id);
                    self.evaluate_integer((value as u128).into(), false, id)
                } else {
                    let location = self.elaborator.interner.expr_location(&id);
                    let typ = Type::TypeVariable(type_variable.clone(), TypeVariableKind::Normal);
                    Err(InterpreterError::NonIntegerArrayLength { typ, location })
                }
            }
        }
    }

    fn evaluate_literal(&mut self, literal: HirLiteral, id: ExprId) -> IResult<Value> {
        match literal {
            HirLiteral::Unit => Ok(Value::Unit),
            HirLiteral::Bool(value) => Ok(Value::Bool(value)),
            HirLiteral::Integer(value, is_negative) => {
                self.evaluate_integer(value, is_negative, id)
            }
            HirLiteral::Str(string) => Ok(Value::String(Rc::new(string))),
            HirLiteral::FmtStr(string, captures) => {
                self.evaluate_format_string(string, captures, id)
            }
            HirLiteral::Array(array) => self.evaluate_array(array, id),
            HirLiteral::Slice(array) => self.evaluate_slice(array, id),
        }
    }

    fn evaluate_format_string(
        &mut self,
        string: String,
        captures: Vec<ExprId>,
        id: ExprId,
    ) -> IResult<Value> {
        let mut result = String::new();
        let mut escaped = false;
        let mut consuming = false;

        let mut values: VecDeque<_> =
            captures.into_iter().map(|capture| self.evaluate(capture)).collect::<Result<_, _>>()?;

        for character in string.chars() {
            match character {
                '\\' => escaped = true,
                '{' if !escaped => consuming = true,
                '}' if !escaped && consuming => {
                    consuming = false;

                    if let Some(value) = values.pop_front() {
                        result.push_str(&value.display(self.elaborator.interner).to_string());
                    }
                }
                other if !consuming => {
                    escaped = false;
                    result.push(other);
                }
                _ => (),
            }
        }

        let typ = self.elaborator.interner.id_type(id);
        Ok(Value::FormatString(Rc::new(result), typ))
    }

    fn evaluate_integer(
        &self,
        value: FieldElement,
        is_negative: bool,
        id: ExprId,
    ) -> IResult<Value> {
        let typ = self.elaborator.interner.id_type(id).follow_bindings();
        let location = self.elaborator.interner.expr_location(&id);

        if let Type::FieldElement = &typ {
            Ok(Value::Field(value))
        } else if let Type::Integer(sign, bit_size) = &typ {
            match (sign, bit_size) {
                (Signedness::Unsigned, IntegerBitSize::One) => {
                    return Err(InterpreterError::TypeUnsupported { typ, location });
                }
                (Signedness::Unsigned, IntegerBitSize::Eight) => {
                    let value: u8 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { 0u8.wrapping_sub(value) } else { value };
                    Ok(Value::U8(value))
                }
                (Signedness::Unsigned, IntegerBitSize::Sixteen) => {
                    let value: u16 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { 0u16.wrapping_sub(value) } else { value };
                    Ok(Value::U16(value))
                }
                (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                    let value: u32 =
                        value.try_to_u32().ok_or(InterpreterError::IntegerOutOfRangeForType {
                            value,
                            typ,
                            location,
                        })?;
                    let value = if is_negative { 0u32.wrapping_sub(value) } else { value };
                    Ok(Value::U32(value))
                }
                (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                    let value: u64 =
                        value.try_to_u64().ok_or(InterpreterError::IntegerOutOfRangeForType {
                            value,
                            typ,
                            location,
                        })?;
                    let value = if is_negative { 0u64.wrapping_sub(value) } else { value };
                    Ok(Value::U64(value))
                }
                (Signedness::Signed, IntegerBitSize::One) => {
                    return Err(InterpreterError::TypeUnsupported { typ, location });
                }
                (Signedness::Signed, IntegerBitSize::Eight) => {
                    let value: i8 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I8(value))
                }
                (Signedness::Signed, IntegerBitSize::Sixteen) => {
                    let value: i16 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I16(value))
                }
                (Signedness::Signed, IntegerBitSize::ThirtyTwo) => {
                    let value: i32 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I32(value))
                }
                (Signedness::Signed, IntegerBitSize::SixtyFour) => {
                    let value: i64 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I64(value))
                }
            }
        } else if let Type::TypeVariable(variable, TypeVariableKind::IntegerOrField) = &typ {
            Ok(Value::Field(value))
        } else if let Type::TypeVariable(variable, TypeVariableKind::Integer) = &typ {
            let value: u64 = value
                .try_to_u64()
                .ok_or(InterpreterError::IntegerOutOfRangeForType { value, typ, location })?;
            let value = if is_negative { 0u64.wrapping_sub(value) } else { value };
            Ok(Value::U64(value))
        } else {
            Err(InterpreterError::NonIntegerIntegerLiteral { typ, location })
        }
    }

    pub fn evaluate_block(&mut self, mut block: HirBlockExpression) -> IResult<Value> {
        let last_statement = block.statements.pop();
        self.push_scope();

        for statement in block.statements {
            self.evaluate_statement(statement)?;
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

                if let Some(length) = length.evaluate_to_u32() {
                    let elements = (0..length).map(|_| element.clone()).collect();
                    Ok(Value::Array(elements, typ))
                } else {
                    let location = self.elaborator.interner.expr_location(&id);
                    Err(InterpreterError::NonIntegerArrayLength { typ: length, location })
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
            UnaryOp::MutableReference => self.evaluate_no_dereference(prefix.rhs)?,
            _ => self.evaluate(prefix.rhs)?,
        };

        if self.elaborator.interner.get_selected_impl_for_expression(id).is_some() {
            self.evaluate_overloaded_prefix(prefix, rhs, id)
        } else {
            self.evaluate_prefix_with_value(rhs, prefix.operator, id)
        }
    }

    fn evaluate_prefix_with_value(
        &mut self,
        rhs: Value,
        operator: UnaryOp,
        id: ExprId,
    ) -> IResult<Value> {
        match operator {
            UnaryOp::Minus => match rhs {
                Value::Field(value) => Ok(Value::Field(FieldElement::zero() - value)),
                Value::I8(value) => Ok(Value::I8(-value)),
                Value::I16(value) => Ok(Value::I16(-value)),
                Value::I32(value) => Ok(Value::I32(-value)),
                Value::I64(value) => Ok(Value::I64(-value)),
                Value::U8(value) => Ok(Value::U8(0 - value)),
                Value::U16(value) => Ok(Value::U16(0 - value)),
                Value::U32(value) => Ok(Value::U32(0 - value)),
                Value::U64(value) => Ok(Value::U64(0 - value)),
                value => {
                    let location = self.elaborator.interner.expr_location(&id);
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
                Value::U8(value) => Ok(Value::U8(!value)),
                Value::U16(value) => Ok(Value::U16(!value)),
                Value::U32(value) => Ok(Value::U32(!value)),
                Value::U64(value) => Ok(Value::U64(!value)),
                value => {
                    let location = self.elaborator.interner.expr_location(&id);
                    let typ = value.get_type().into_owned();
                    Err(InterpreterError::InvalidValueForUnary { typ, location, operator: "not" })
                }
            },
            UnaryOp::MutableReference => {
                // If this is a mutable variable (auto_deref = true), turn this into an explicit
                // mutable reference just by switching the value of `auto_deref`. Otherwise, wrap
                // the value in a fresh reference.
                match rhs {
                    Value::Pointer(elem, true) => Ok(Value::Pointer(elem, false)),
                    other => Ok(Value::Pointer(Shared::new(other), false)),
                }
            }
            UnaryOp::Dereference { implicitly_added: _ } => match rhs {
                Value::Pointer(element, _) => Ok(element.borrow().clone()),
                value => {
                    let location = self.elaborator.interner.expr_location(&id);
                    let typ = value.get_type().into_owned();
                    Err(InterpreterError::NonPointerDereferenced { typ, location })
                }
            },
        }
    }

    fn evaluate_infix(&mut self, infix: HirInfixExpression, id: ExprId) -> IResult<Value> {
        let lhs = self.evaluate(infix.lhs)?;
        let rhs = self.evaluate(infix.rhs)?;

        if self.elaborator.interner.get_selected_impl_for_expression(id).is_some() {
            return self.evaluate_overloaded_infix(infix, lhs, rhs, id);
        }

        let make_error = |this: &mut Self, lhs: Value, rhs: Value, operator| {
            let location = this.elaborator.interner.expr_location(&id);
            let lhs = lhs.get_type().into_owned();
            let rhs = rhs.get_type().into_owned();
            Err(InvalidValuesForBinary { lhs, rhs, location, operator })
        };

        use InterpreterError::InvalidValuesForBinary;
        match infix.operator.kind {
            BinaryOpKind::Add => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs + rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs + rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs + rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs + rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs + rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs + rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs + rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs + rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs + rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "+"),
            },
            BinaryOpKind::Subtract => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs - rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs - rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs - rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs - rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs - rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs - rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs - rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs - rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs - rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "-"),
            },
            BinaryOpKind::Multiply => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs * rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs * rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs * rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs * rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs * rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs * rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs * rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs * rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs * rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "*"),
            },
            BinaryOpKind::Divide => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs / rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs / rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs / rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs / rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs / rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs / rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs / rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs / rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs / rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "/"),
            },
            BinaryOpKind::Equal => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "=="),
            },
            BinaryOpKind::NotEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "!="),
            },
            BinaryOpKind::Less => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "<"),
            },
            BinaryOpKind::LessEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "<="),
            },
            BinaryOpKind::Greater => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, ">"),
            },
            BinaryOpKind::GreaterEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, ">="),
            },
            BinaryOpKind::And => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs & rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs & rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs & rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs & rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs & rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs & rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs & rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs & rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs & rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "&"),
            },
            BinaryOpKind::Or => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs | rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs | rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs | rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs | rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs | rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs | rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs | rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs | rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs | rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "|"),
            },
            BinaryOpKind::Xor => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs ^ rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs ^ rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs ^ rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs ^ rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs ^ rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs ^ rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs ^ rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs ^ rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs ^ rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "^"),
            },
            BinaryOpKind::ShiftRight => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs >> rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs >> rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs >> rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs >> rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs >> rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs >> rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs >> rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs >> rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, ">>"),
            },
            BinaryOpKind::ShiftLeft => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs << rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs << rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs << rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs << rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs << rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs << rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs << rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs << rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "<<"),
            },
            BinaryOpKind::Modulo => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs % rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs % rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs % rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs % rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs % rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs % rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs % rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs % rhs)),
                (lhs, rhs) => make_error(self, lhs, rhs, "%"),
            },
        }
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

        let method_id = resolve_trait_method(self.elaborator.interner, method, id)?;
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
            NotEqual => self.evaluate_prefix_with_value(value, UnaryOp::Not, id),
            Less | LessEqual | Greater | GreaterEqual => self.evaluate_ordering(value, operator),
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
        let operator = prefix.operator;

        let method_id = resolve_trait_method(self.elaborator.interner, method, id)?;
        let type_bindings = self.elaborator.interner.get_instantiation_bindings(id).clone();

        let rhs = (rhs, self.elaborator.interner.expr_location(&prefix.rhs));

        let location = self.elaborator.interner.expr_location(&id);
        self.call_function(method_id, vec![rhs], type_bindings, location)
    }

    /// Given the result of a `cmp` operation, convert it into the boolean result of the given operator.
    /// - `<`:  `ordering == Ordering::Less`
    /// - `<=`: `ordering != Ordering::Greater`
    /// - `>`:  `ordering == Ordering::Greater`
    /// - `<=`: `ordering != Ordering::Less`
    fn evaluate_ordering(&self, ordering: Value, operator: BinaryOpKind) -> IResult<Value> {
        let ordering = match ordering {
            Value::Struct(fields, _) => match fields.into_iter().next().unwrap().1 {
                Value::Field(ordering) => ordering,
                _ => unreachable!("`cmp` should always return an Ordering value"),
            },
            _ => unreachable!("`cmp` should always return an Ordering value"),
        };

        use BinaryOpKind::*;
        let less_or_greater = if matches!(operator, Less | GreaterEqual) {
            FieldElement::zero() // Ordering::Less
        } else {
            2u128.into() // Ordering::Greater
        };

        if matches!(operator, Less | Greater) {
            Ok(Value::Bool(ordering == less_or_greater))
        } else {
            Ok(Value::Bool(ordering != less_or_greater))
        }
    }

    fn evaluate_index(&mut self, index: HirIndexExpression, id: ExprId) -> IResult<Value> {
        let array = self.evaluate(index.collection)?;
        let index = self.evaluate(index.index)?;

        let location = self.elaborator.interner.expr_location(&id);
        let (array, index) = self.bounds_check(array, index, location)?;

        Ok(array[index].clone())
    }

    /// Bounds check the given array and index pair.
    /// This will also ensure the given arguments are in fact an array and integer.
    fn bounds_check(
        &self,
        array: Value,
        index: Value,
        location: Location,
    ) -> IResult<(Vector<Value>, usize)> {
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
                value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or_else(|| {
                    let typ = Type::default_int_type();
                    InterpreterError::IntegerOutOfRangeForType { value, typ, location }
                })?
            }
            Value::I8(value) => value as usize,
            Value::I16(value) => value as usize,
            Value::I32(value) => value as usize,
            Value::I64(value) => value as usize,
            Value::U8(value) => value as usize,
            Value::U16(value) => value as usize,
            Value::U32(value) => value as usize,
            Value::U64(value) => value as usize,
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

    fn evaluate_constructor(
        &mut self,
        constructor: HirConstructorExpression,
        id: ExprId,
    ) -> IResult<Value> {
        let fields = constructor
            .fields
            .into_iter()
            .map(|(name, expr)| {
                let field_value = self.evaluate(expr)?;
                Ok((Rc::new(name.0.contents), field_value))
            })
            .collect::<Result<_, _>>()?;

        let typ = self.elaborator.interner.id_type(id).follow_bindings();
        Ok(Value::Struct(fields, typ))
    }

    fn evaluate_access(&mut self, access: HirMemberAccess, id: ExprId) -> IResult<Value> {
        let (fields, struct_type) = match self.evaluate(access.lhs)? {
            Value::Struct(fields, typ) => (fields, typ),
            Value::Tuple(fields) => {
                let (fields, field_types): (HashMap<Rc<String>, Value>, Vec<Type>) = fields
                    .into_iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let field_type = field.get_type().into_owned();
                        let key_val_pair = (Rc::new(i.to_string()), field);
                        (key_val_pair, field_type)
                    })
                    .unzip();
                (fields, Type::Tuple(field_types))
            }
            value => {
                let location = self.elaborator.interner.expr_location(&id);
                let typ = value.get_type().into_owned();
                return Err(InterpreterError::NonTupleOrStructInMemberAccess { typ, location });
            }
        };

        fields.get(&access.rhs.0.contents).cloned().ok_or_else(|| {
            let location = self.elaborator.interner.expr_location(&id);
            let value = Value::Struct(fields, struct_type);
            let field_name = access.rhs.0.contents;
            let typ = value.get_type().into_owned();
            InterpreterError::ExpectedStructToHaveField { typ, field_name, location }
        })
    }

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
                    let expr = result.into_expression(self.elaborator.interner, location)?;
                    let expr = self.elaborate_item(self.current_function, |elaborator| {
                        elaborator.elaborate_expression(expr).0
                    });
                    result = self.evaluate(expr)?;
                }
                Ok(result)
            }
            Value::Closure(closure, env, _) => self.call_closure(closure, env, arguments, location),
            value => {
                let typ = value.get_type().into_owned();
                Err(InterpreterError::NonFunctionCalled { typ, location })
            }
        }
    }

    fn evaluate_method_call(
        &mut self,
        call: HirMethodCallExpression,
        id: ExprId,
    ) -> IResult<Value> {
        let object = self.evaluate(call.object)?;
        let arguments = try_vecmap(call.arguments, |arg| {
            Ok((self.evaluate(arg)?, self.elaborator.interner.expr_location(&arg)))
        })?;
        let location = self.elaborator.interner.expr_location(&id);

        let typ = object.get_type().follow_bindings();
        let method_name = &call.method.0.contents;

        // TODO: Traits
        let method = match &typ {
            Type::Struct(struct_def, _) => self.elaborator.interner.lookup_method(
                &typ,
                struct_def.borrow().id,
                method_name,
                false,
            ),
            _ => self.elaborator.interner.lookup_primitive_method(&typ, method_name),
        };

        if let Some(method) = method {
            self.call_function(method, arguments, TypeBindings::new(), location)
        } else {
            Err(InterpreterError::NoMethodFound { name: method_name.clone(), typ, location })
        }
    }

    fn evaluate_cast(&mut self, cast: &HirCastExpression, id: ExprId) -> IResult<Value> {
        let evaluated_lhs = self.evaluate(cast.lhs)?;
        Self::evaluate_cast_one_step(cast, id, evaluated_lhs, self.elaborator.interner)
    }

    /// evaluate_cast without recursion
    pub fn evaluate_cast_one_step(
        cast: &HirCastExpression,
        id: ExprId,
        evaluated_lhs: Value,
        interner: &NodeInterner,
    ) -> IResult<Value> {
        macro_rules! signed_int_to_field {
            ($x:expr) => {{
                // Need to convert the signed integer to an i128 before
                // we negate it to preserve the MIN value.
                let mut value = $x as i128;
                let is_negative = value < 0;
                if is_negative {
                    value = -value;
                }
                ((value as u128).into(), is_negative)
            }};
        }

        let (mut lhs, lhs_is_negative) = match evaluated_lhs {
            Value::Field(value) => (value, false),
            Value::U8(value) => ((value as u128).into(), false),
            Value::U16(value) => ((value as u128).into(), false),
            Value::U32(value) => ((value as u128).into(), false),
            Value::U64(value) => ((value as u128).into(), false),
            Value::I8(value) => signed_int_to_field!(value),
            Value::I16(value) => signed_int_to_field!(value),
            Value::I32(value) => signed_int_to_field!(value),
            Value::I64(value) => signed_int_to_field!(value),
            Value::Bool(value) => {
                (if value { FieldElement::one() } else { FieldElement::zero() }, false)
            }
            value => {
                let location = interner.expr_location(&id);
                let typ = value.get_type().into_owned();
                return Err(InterpreterError::NonNumericCasted { typ, location });
            }
        };

        macro_rules! cast_to_int {
            ($x:expr, $method:ident, $typ:ty, $f:ident) => {{
                let mut value = $x.$method() as $typ;
                if lhs_is_negative {
                    value = 0 - value;
                }
                Ok(Value::$f(value))
            }};
        }

        // Now actually cast the lhs, bit casting and wrapping as necessary
        match cast.r#type.follow_bindings() {
            Type::FieldElement => {
                if lhs_is_negative {
                    lhs = FieldElement::zero() - lhs;
                }
                Ok(Value::Field(lhs))
            }
            Type::Integer(sign, bit_size) => match (sign, bit_size) {
                (Signedness::Unsigned, IntegerBitSize::One) => {
                    let location = interner.expr_location(&id);
                    Err(InterpreterError::TypeUnsupported { typ: cast.r#type.clone(), location })
                }
                (Signedness::Unsigned, IntegerBitSize::Eight) => cast_to_int!(lhs, to_u128, u8, U8),
                (Signedness::Unsigned, IntegerBitSize::Sixteen) => {
                    cast_to_int!(lhs, to_u128, u16, U16)
                }
                (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                    cast_to_int!(lhs, to_u128, u32, U32)
                }
                (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                    cast_to_int!(lhs, to_u128, u64, U64)
                }
                (Signedness::Signed, IntegerBitSize::One) => {
                    let location = interner.expr_location(&id);
                    Err(InterpreterError::TypeUnsupported { typ: cast.r#type.clone(), location })
                }
                (Signedness::Signed, IntegerBitSize::Eight) => cast_to_int!(lhs, to_i128, i8, I8),
                (Signedness::Signed, IntegerBitSize::Sixteen) => {
                    cast_to_int!(lhs, to_i128, i16, I16)
                }
                (Signedness::Signed, IntegerBitSize::ThirtyTwo) => {
                    cast_to_int!(lhs, to_i128, i32, I32)
                }
                (Signedness::Signed, IntegerBitSize::SixtyFour) => {
                    cast_to_int!(lhs, to_i128, i64, I64)
                }
            },
            Type::Bool => Ok(Value::Bool(!lhs.is_zero() || lhs_is_negative)),
            typ => {
                let location = interner.expr_location(&id);
                Err(InterpreterError::CastToNonNumericType { typ, location })
            }
        }
    }

    fn evaluate_if(&mut self, if_: HirIfExpression, id: ExprId) -> IResult<Value> {
        let condition = match self.evaluate(if_.condition)? {
            Value::Bool(value) => value,
            value => {
                let location = self.elaborator.interner.expr_location(&id);
                let typ = value.get_type().into_owned();
                return Err(InterpreterError::NonBoolUsedInIf { typ, location });
            }
        };

        self.push_scope();

        let result = if condition {
            if if_.alternative.is_some() {
                self.evaluate(if_.consequence)
            } else {
                self.evaluate(if_.consequence)?;
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
        let fields = try_vecmap(tuple, |field| self.evaluate(field))?;
        Ok(Value::Tuple(fields))
    }

    fn evaluate_lambda(&mut self, lambda: HirLambda, id: ExprId) -> IResult<Value> {
        let location = self.elaborator.interner.expr_location(&id);
        let environment =
            try_vecmap(&lambda.captures, |capture| self.lookup_id(capture.ident.id, location))?;

        let typ = self.elaborator.interner.id_type(id).follow_bindings();
        Ok(Value::Closure(lambda, environment, typ))
    }

    fn evaluate_quote(&mut self, mut tokens: Tokens, expr_id: ExprId) -> IResult<Value> {
        let location = self.elaborator.interner.expr_location(&expr_id);
        let tokens = self.substitute_unquoted_values_into_tokens(tokens, location)?;
        Ok(Value::Quoted(Rc::new(tokens)))
    }

    pub fn evaluate_statement(&mut self, statement: StmtId) -> IResult<Value> {
        match self.elaborator.interner.statement(&statement) {
            HirStatement::Let(let_) => self.evaluate_let(let_),
            HirStatement::Constrain(constrain) => self.evaluate_constrain(constrain),
            HirStatement::Assign(assign) => self.evaluate_assign(assign),
            HirStatement::For(for_) => self.evaluate_for(for_),
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

    pub fn evaluate_let(&mut self, let_: HirLetStatement) -> IResult<Value> {
        let rhs = self.evaluate(let_.expression)?;
        let location = self.elaborator.interner.expr_location(&let_.expression);
        self.define_pattern(&let_.pattern, &let_.r#type, rhs, location)?;
        Ok(Value::Unit)
    }

    fn evaluate_constrain(&mut self, constrain: HirConstrainStatement) -> IResult<Value> {
        match self.evaluate(constrain.0)? {
            Value::Bool(true) => Ok(Value::Unit),
            Value::Bool(false) => {
                let location = self.elaborator.interner.expr_location(&constrain.0);
                let message = constrain.2.and_then(|expr| self.evaluate(expr).ok());
                let message =
                    message.map(|value| value.display(self.elaborator.interner).to_string());
                Err(InterpreterError::FailingConstraint { location, message })
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

    fn store_lvalue(&mut self, lvalue: HirLValue, rhs: Value) -> IResult<()> {
        match lvalue {
            HirLValue::Ident(ident, typ) => self.mutate(ident.id, rhs, ident.location),
            HirLValue::Dereference { lvalue, element_type: _, location } => {
                match self.evaluate_lvalue(&lvalue)? {
                    Value::Pointer(value, _) => {
                        *value.borrow_mut() = rhs;
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
                        fields[index] = rhs;
                        self.store_lvalue(*object, Value::Tuple(fields))
                    }
                    Value::Struct(mut fields, typ) => {
                        fields.insert(Rc::new(field_name.0.contents), rhs);
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
                let (elements, index) = self.bounds_check(array_value, index, location)?;

                let new_array = constructor(elements.update(index, rhs), typ);
                self.store_lvalue(*array, new_array)
            }
        }
    }

    fn evaluate_lvalue(&mut self, lvalue: &HirLValue) -> IResult<Value> {
        match lvalue {
            HirLValue::Ident(ident, _) => match self.lookup(ident)? {
                Value::Pointer(elem, true) => Ok(elem.borrow().clone()),
                other => Ok(other),
            },
            HirLValue::Dereference { lvalue, element_type: _, location } => {
                match self.evaluate_lvalue(lvalue)? {
                    Value::Pointer(value, _) => Ok(value.borrow().clone()),
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
                    Value::Tuple(mut values) => Ok(values.swap_remove(index)),
                    Value::Struct(fields, _) => Ok(fields[&field_name.0.contents].clone()),
                    value => Err(InterpreterError::NonTupleOrStructInMemberAccess {
                        typ: value.get_type().into_owned(),
                        location: *location,
                    }),
                }
            }
            HirLValue::Index { array, index, typ: _, location } => {
                let array = self.evaluate_lvalue(array)?;
                let index = self.evaluate(*index)?;
                let (elements, index) = self.bounds_check(array, index, *location)?;
                Ok(elements[index].clone())
            }
        }
    }

    fn evaluate_for(&mut self, for_: HirForStatement) -> IResult<Value> {
        // i128 can store all values from i8 - u64
        let get_index = |this: &mut Self, expr| -> IResult<(_, fn(_) -> _)> {
            match this.evaluate(expr)? {
                Value::I8(value) => Ok((value as i128, |i| Value::I8(i as i8))),
                Value::I16(value) => Ok((value as i128, |i| Value::I16(i as i16))),
                Value::I32(value) => Ok((value as i128, |i| Value::I32(i as i32))),
                Value::I64(value) => Ok((value as i128, |i| Value::I64(i as i64))),
                Value::U8(value) => Ok((value as i128, |i| Value::U8(i as u8))),
                Value::U16(value) => Ok((value as i128, |i| Value::U16(i as u16))),
                Value::U32(value) => Ok((value as i128, |i| Value::U32(i as u32))),
                Value::U64(value) => Ok((value as i128, |i| Value::U64(i as u64))),
                value => {
                    let location = this.elaborator.interner.expr_location(&expr);
                    let typ = value.get_type().into_owned();
                    Err(InterpreterError::NonIntegerUsedInLoop { typ, location })
                }
            }
        };

        let (start, make_value) = get_index(self, for_.start_range)?;
        let (end, _) = get_index(self, for_.end_range)?;
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);

        for i in start..end {
            self.push_scope();
            self.current_scope_mut().insert(for_.identifier.id, make_value(i));

            match self.evaluate(for_.block) {
                Ok(_) => (),
                Err(InterpreterError::Break) => break,
                Err(InterpreterError::Continue) => continue,
                Err(other) => return Err(other),
            }

            self.pop_scope();
        }

        self.in_loop = was_in_loop;
        Ok(Value::Unit)
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

    fn print_oracle(&self, arguments: Vec<(Value, Location)>) -> Result<Value, InterpreterError> {
        assert_eq!(arguments.len(), 2);

        let print_newline = arguments[0].0 == Value::Bool(true);
        if print_newline {
            println!("{}", arguments[1].0.display(self.elaborator.interner));
        } else {
            print!("{}", arguments[1].0.display(self.elaborator.interner));
        }

        Ok(Value::Unit)
    }
}
