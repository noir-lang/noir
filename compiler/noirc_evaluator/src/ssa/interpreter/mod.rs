use std::cmp::Ordering;

use super::{
    Ssa,
    ir::{
        dfg::DataFlowGraph,
        function::{Function, FunctionId, RuntimeType},
        instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
        types::Type,
        value::ValueId,
    },
};
use crate::{errors::RuntimeError, ssa::ir::instruction::binary::truncate_field};
use acvm::{AcirField, FieldElement};
use fxhash::FxHashMap as HashMap;
use iter_extended::vecmap;
use noirc_frontend::Shared;
use value::{ArrayValue, NumericValue};

mod intrinsics;
mod tests;
pub mod value;

use value::Value;

struct Interpreter<'ssa> {
    /// Contains each function called with `main` (or the first called function if
    /// the interpreter was manually invoked on a different function) at
    /// the front of the Vec.
    call_stack: Vec<CallContext>,

    ssa: &'ssa Ssa,

    /// This variable can be modified by `enable_side_effects_if` instructions and is
    /// expected to have no effect if there are no such instructions or if the code
    /// being executed is an unconstrained function.
    side_effects_enabled: bool,
}

struct CallContext {
    /// The function that was called. This is `None` only for the top-level global
    /// scope where global instructions are evaluated.
    called_function: Option<FunctionId>,

    /// Contains each value currently defined and visible to the current function.
    scope: HashMap<ValueId, Value>,
}

impl CallContext {
    fn new(called_function: FunctionId) -> Self {
        Self { called_function: Some(called_function), scope: Default::default() }
    }

    fn global_context() -> Self {
        Self { called_function: None, scope: Default::default() }
    }
}

type IResult<T> = Result<T, RuntimeError>;
type IResults = IResult<Vec<Value>>;

#[allow(unused)]
impl Ssa {
    pub(crate) fn interpret(&self, args: Vec<Value>) -> IResults {
        self.interpret_function(self.main_id, args)
    }

    pub(crate) fn interpret_function(&self, function: FunctionId, args: Vec<Value>) -> IResults {
        let mut interpreter = Interpreter::new(self);
        interpreter.interpret_globals()?;
        interpreter.call_function(function, args)
    }
}

impl<'ssa> Interpreter<'ssa> {
    fn new(ssa: &'ssa Ssa) -> Self {
        let call_stack = vec![CallContext::global_context()];
        Self { ssa, call_stack, side_effects_enabled: true }
    }

    fn call_context(&self) -> &CallContext {
        self.call_stack.last().expect("call_stack should always be non-empty")
    }

    fn call_context_mut(&mut self) -> &mut CallContext {
        self.call_stack.last_mut().expect("call_stack should always be non-empty")
    }

    fn global_scope(&self) -> &HashMap<ValueId, Value> {
        &self.call_stack.first().expect("call_stack should always be non-empty").scope
    }

    fn current_function(&self) -> &'ssa Function {
        let current_function_id = self.call_context().called_function;
        let current_function_id = current_function_id.expect(
            "Tried calling `Interpreter::current_function` while evaluating global instructions",
        );
        &self.ssa.functions[&current_function_id]
    }

    fn dfg(&self) -> &'ssa DataFlowGraph {
        &self.current_function().dfg
    }

    fn in_unconstrained_context(&self) -> bool {
        self.current_function().runtime().is_brillig()
    }

    /// Define or redefine a value.
    /// Redefinitions are expected in the case of loops.
    fn define(&mut self, id: ValueId, value: Value) {
        self.call_context_mut().scope.insert(id, value);
    }

    fn interpret_globals(&mut self) -> IResult<()> {
        let globals = &self.ssa.main().dfg.globals;
        for (global_id, global) in globals.values_iter() {
            let value = match dbg!(global) {
                super::ir::value::Value::Instruction { instruction, .. } => {
                    let instruction = &globals[*instruction];
                    self.interpret_instruction(instruction, &[global_id])?;
                    continue;
                }
                super::ir::value::Value::NumericConstant { constant, typ } => {
                    Value::from_constant(*constant, *typ)
                }
                super::ir::value::Value::Function(id) => Value::Function(*id),
                super::ir::value::Value::Intrinsic(intrinsic) => Value::Intrinsic(*intrinsic),
                super::ir::value::Value::ForeignFunction(name) => {
                    Value::ForeignFunction(name.clone())
                }
                super::ir::value::Value::Global(_) | super::ir::value::Value::Param { .. } => {
                    unreachable!()
                }
            };
            self.define(global_id, value);
        }
        Ok(())
    }

    fn call_function(&mut self, function_id: FunctionId, mut arguments: Vec<Value>) -> IResults {
        self.call_stack.push(CallContext::new(function_id));

        let function = &self.ssa.functions[&function_id];
        let mut block_id = function.entry_block();
        let dfg = self.dfg();

        // Loop over blocks & instructions inline here to avoid pushing more
        // call frames (in rust). We only push call frames for function calls which
        // should prevent stack overflows for all but excessively large call stacks
        // that may overflow in the brillig vm as well.
        let return_values = loop {
            let block = &dfg[block_id];

            if arguments.len() != block.parameters().len() {
                panic!("Block argument count does not match the expected parameter count");
            }

            for (parameter, argument) in block.parameters().iter().zip(arguments) {
                self.define(*parameter, argument);
            }

            for instruction_id in block.instructions() {
                let results = dfg.instruction_results(*instruction_id);
                self.interpret_instruction(&dfg[*instruction_id], results)?;
            }

            match block.terminator() {
                None => panic!("No block terminator in block {block_id}"),
                Some(TerminatorInstruction::Jmp { destination, arguments: jump_args, .. }) => {
                    block_id = *destination;
                    arguments = self.lookup_all(jump_args);
                }
                Some(TerminatorInstruction::JmpIf {
                    condition,
                    then_destination,
                    else_destination,
                    call_stack: _,
                }) => {
                    block_id = if self.lookup(*condition).as_bool().unwrap() {
                        *then_destination
                    } else {
                        *else_destination
                    };
                    arguments = Vec::new();
                }
                Some(TerminatorInstruction::Return { return_values, call_stack: _ }) => {
                    break self.lookup_all(return_values);
                }
            }
        };

        self.call_stack.pop();
        Ok(return_values)
    }

    fn lookup(&self, id: ValueId) -> Value {
        if let Some(value) = self.call_context().scope.get(&id) {
            return value.clone();
        }

        if let Some(value) = self.global_scope().get(&id) {
            return value.clone();
        }

        match &self.dfg()[id] {
            super::ir::value::Value::NumericConstant { constant, typ } => {
                Value::from_constant(*constant, *typ)
            }
            super::ir::value::Value::Function(id) => Value::Function(*id),
            super::ir::value::Value::Intrinsic(intrinsic) => Value::Intrinsic(*intrinsic),
            super::ir::value::Value::ForeignFunction(name) => Value::ForeignFunction(name.clone()),
            super::ir::value::Value::Instruction { .. }
            | super::ir::value::Value::Param { .. }
            | super::ir::value::Value::Global(_) => {
                unreachable!("`{id}` should already be in scope")
            }
        }
    }

    fn lookup_all(&self, ids: &[ValueId]) -> Vec<Value> {
        vecmap(ids, |id| self.lookup(*id))
    }

    fn side_effects_enabled(&self) -> bool {
        match self.current_function().runtime() {
            RuntimeType::Acir(_) => self.side_effects_enabled,
            RuntimeType::Brillig(_) => true,
        }
    }

    #[allow(unused)]
    fn interpret_instruction(
        &mut self,
        instruction: &Instruction,
        results: &[ValueId],
    ) -> Result<(), RuntimeError> {
        match instruction {
            Instruction::Binary(binary) => {
                let result = self.interpret_binary(binary)?;
                self.define(results[0], result);
            }
            // Cast in SSA changes the type without altering the value
            Instruction::Cast(value, numeric_type) => {
                let field = self.lookup(*value).as_numeric().unwrap().convert_to_field();
                let result = Value::Numeric(NumericValue::from_constant(field, *numeric_type));
                self.define(results[0], result);
            }
            Instruction::Not(id) => self.interpret_not(*id, results[0]),
            Instruction::Truncate { value, bit_size, max_bit_size } => {
                self.interpret_truncate(*value, *bit_size, *max_bit_size, results[0]);
            }
            Instruction::Constrain(lhs, rhs, constrain_error) => {
                let lhs = self.lookup(*lhs);
                let rhs = self.lookup(*rhs);
                if self.side_effects_enabled() && lhs != rhs {
                    panic!("Constrain {lhs} == {rhs} failed!");
                }
            }
            Instruction::ConstrainNotEqual(lhs, rhs, constrain_error) => {
                let lhs = self.lookup(*lhs);
                let rhs = self.lookup(*rhs);
                if self.side_effects_enabled() && lhs == rhs {
                    panic!("Constrain {lhs} != {rhs} failed!");
                }
            }
            Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                self.interpret_range_check(*value, *max_bit_size, assert_message.as_ref());
            }
            Instruction::Call { func, arguments } => {
                self.interpret_call(*func, arguments, results)?;
            }
            Instruction::Allocate => self.interpret_allocate(results[0]),
            Instruction::Load { address } => self.interpret_load(*address, results[0]),
            Instruction::Store { address, value } => self.interpret_store(*address, *value),
            Instruction::EnableSideEffectsIf { condition } => {
                self.side_effects_enabled = self.lookup(*condition).as_bool().unwrap();
            }
            Instruction::ArrayGet { array, index } => {
                self.interpret_array_get(*array, *index, results[0]);
            }
            Instruction::ArraySet { array, index, value, mutable } => {
                self.interpret_array_set(*array, *index, *value, *mutable, results[0]);
            }
            Instruction::IncrementRc { value } => self.interpret_inc_rc(*value),
            Instruction::DecrementRc { value } => self.interpret_dec_rc(*value),
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => self
                .interpret_if_else(
                    *then_condition,
                    *then_value,
                    *else_condition,
                    *else_value,
                    results[0],
                ),
            Instruction::MakeArray { elements, typ } => {
                self.interpret_make_array(elements, results[0], typ);
            }
            Instruction::Noop => (),
        }
        Ok(())
    }

    fn interpret_not(&mut self, id: ValueId, result: ValueId) {
        let new_result = match self.lookup(id).as_numeric().unwrap() {
            NumericValue::Field(field) => {
                unreachable!("not: Expected integer value, found field {field}")
            }
            NumericValue::U1(value) => NumericValue::U1(!value),
            NumericValue::U8(value) => NumericValue::U8(!value),
            NumericValue::U16(value) => NumericValue::U16(!value),
            NumericValue::U32(value) => NumericValue::U32(!value),
            NumericValue::U64(value) => NumericValue::U64(!value),
            NumericValue::U128(value) => NumericValue::U128(!value),
            NumericValue::I8(value) => NumericValue::I8(!value),
            NumericValue::I16(value) => NumericValue::I16(!value),
            NumericValue::I32(value) => NumericValue::I32(!value),
            NumericValue::I64(value) => NumericValue::I64(!value),
        };
        self.define(result, Value::Numeric(new_result));
    }

    fn interpret_truncate(
        &mut self,
        value: ValueId,
        bit_size: u32,
        _max_bit_size: u32,
        result: ValueId,
    ) {
        let value = self.lookup(value).as_numeric().unwrap();
        let bit_mask = (1u128 << bit_size) - 1;
        assert_ne!(bit_mask, 0);

        let truncated = match value {
            NumericValue::Field(value) => NumericValue::Field(truncate_field(value, bit_size)),
            NumericValue::U1(value) => NumericValue::U1(value),
            NumericValue::U8(value) => NumericValue::U8(truncate_unsigned(value, bit_size)),
            NumericValue::U16(value) => NumericValue::U16(truncate_unsigned(value, bit_size)),
            NumericValue::U32(value) => NumericValue::U32(truncate_unsigned(value, bit_size)),
            NumericValue::U64(value) => NumericValue::U64(truncate_unsigned(value, bit_size)),
            NumericValue::U128(value) => NumericValue::U128(truncate_unsigned(value, bit_size)),
            NumericValue::I8(value) => NumericValue::I8(truncate_signed(value, bit_size)),
            NumericValue::I16(value) => NumericValue::I16(truncate_signed(value, bit_size)),
            NumericValue::I32(value) => NumericValue::I32(truncate_signed(value, bit_size)),
            NumericValue::I64(value) => NumericValue::I64(truncate_signed(value, bit_size)),
        };

        self.define(result, Value::Numeric(truncated));
    }

    fn interpret_range_check(
        &mut self,
        value: ValueId,
        max_bit_size: u32,
        error_message: Option<&String>,
    ) {
        if !self.side_effects_enabled() {
            return;
        }

        let value = self.lookup(value).as_numeric().unwrap();
        assert_ne!(max_bit_size, 0);

        fn bit_count(x: impl Into<f64>) -> u32 {
            let x = x.into();
            if x <= 0.0001 { 0 } else { x.log2() as u32 + 1 }
        }

        let bit_count = match value {
            NumericValue::Field(value) => value.num_bits(),
            // max_bit_size > 0 so u1 should always pass these checks
            NumericValue::U1(_) => return,
            NumericValue::U8(value) => bit_count(value),
            NumericValue::U16(value) => bit_count(value),
            NumericValue::U32(value) => bit_count(value),
            NumericValue::U64(value) => {
                // u64, u128, and i64 don't impl Into<f64>
                if value == 0 { 0 } else { value.ilog2() + 1 }
            }
            NumericValue::U128(value) => {
                if value == 0 {
                    0
                } else {
                    value.ilog2() + 1
                }
            }
            NumericValue::I8(value) => bit_count(value),
            NumericValue::I16(value) => bit_count(value),
            NumericValue::I32(value) => bit_count(value),
            NumericValue::I64(value) => {
                if value == 0 {
                    0
                } else {
                    value.ilog2() + 1
                }
            }
        };

        if bit_count > max_bit_size {
            if let Some(message) = error_message {
                panic!(
                    "bit count of {bit_count} exceeded max bit count of {max_bit_size}\n{message}"
                );
            } else {
                panic!("bit count of {bit_count} exceeded max bit count of {max_bit_size}");
            }
        }
    }

    fn interpret_call(
        &mut self,
        function: ValueId,
        argument_ids: &[ValueId],
        results: &[ValueId],
    ) -> IResult<()> {
        let function = self.lookup(function);
        let mut arguments = vecmap(argument_ids, |argument| self.lookup(*argument));

        let new_results = if self.side_effects_enabled() {
            match function {
                Value::Function(id) => {
                    // If we're crossing a constrained -> unconstrained boundary we have to wipe
                    // any shared mutable fields in our arguments since brillig should conceptually
                    // receive fresh array on each invocation.
                    if !self.in_unconstrained_context()
                        && self.ssa.functions[&id].runtime().is_brillig()
                    {
                        arguments.iter_mut().for_each(Self::reset_array_state);
                    }
                    self.call_function(id, arguments)?
                }
                Value::Intrinsic(intrinsic) => {
                    self.call_intrinsic(intrinsic, arguments, results)?
                }
                Value::ForeignFunction(name) if name == "print" => self.call_print(arguments)?,
                Value::ForeignFunction(name) => {
                    todo!("call: ForeignFunction({name}) is not yet implemented")
                }
                other => panic!("call: Expected function, found {other:?}"),
            }
        } else {
            vecmap(results, |result| {
                let typ = self.dfg().type_of_value(*result);
                Value::uninitialized(&typ, *result)
            })
        };

        assert_eq!(new_results.len(), results.len());
        for (result, new_result) in results.iter().zip(new_results) {
            self.define(*result, new_result);
        }
        Ok(())
    }

    /// Reset the value's `Shared` states in each array within. This is used to mimic each
    /// invocation of the brillig vm receiving fresh values. No matter the history of this value
    /// (e.g. even if they were previously returned from another brillig function) the reference
    /// count should always be 1 and it shouldn't alias any other arrays.
    fn reset_array_state(value: &mut Value) {
        match value {
            Value::Numeric(_)
            | Value::Function(_)
            | Value::Intrinsic(_)
            | Value::ForeignFunction(_) => (),

            Value::Reference(_) => panic!(
                "No reference values are allowed when crossing the constrained -> unconstrained boundary"
            ),

            Value::ArrayOrSlice(array_value) => {
                let mut elements = array_value.elements.borrow().to_vec();
                elements.iter_mut().for_each(Self::reset_array_state);
                array_value.elements = Shared::new(elements);
                array_value.rc = Shared::new(1);
            }
        }
    }

    fn interpret_allocate(&mut self, result: ValueId) {
        let result_type = self.dfg().type_of_value(result);
        let element_type = match result_type {
            Type::Reference(element_type) => element_type,
            other => unreachable!(
                "Result of allocate should always be a reference type, but found {other}"
            ),
        };
        self.define(result, Value::reference(result, element_type));
    }

    fn interpret_load(&mut self, address: ValueId, result: ValueId) {
        let address = self.lookup(address);
        let address = address.as_reference().unwrap();

        let element = address.element.borrow();
        let Some(value) = &*element else {
            panic!(
                "reference value {} is being loaded before it was stored to",
                address.original_id
            );
        };

        self.define(result, value.clone());
    }

    fn interpret_store(&mut self, address: ValueId, value: ValueId) {
        let address = self.lookup(address);
        let address = address.as_reference().unwrap();
        let value = self.lookup(value);
        *address.element.borrow_mut() = Some(value);
    }

    fn interpret_array_get(&mut self, array: ValueId, index: ValueId, result: ValueId) {
        let element = if self.side_effects_enabled() {
            let array = self.lookup(array);
            let array = array.as_array_or_slice().unwrap();
            let index = self.lookup(index).as_u32().unwrap();
            array.elements.borrow()[index as usize].clone()
        } else {
            let typ = self.dfg().type_of_value(result);
            Value::uninitialized(&typ, result)
        };
        self.define(result, element);
    }

    fn interpret_array_set(
        &mut self,
        array: ValueId,
        index: ValueId,
        value: ValueId,
        mutable: bool,
        result: ValueId,
    ) {
        let result_array = if self.side_effects_enabled() {
            let array = self.lookup(array);
            let array = array.as_array_or_slice().unwrap();
            let index = self.lookup(index).as_u32().unwrap();
            let value = self.lookup(value);

            let should_mutate =
                if self.in_unconstrained_context() { *array.rc.borrow() == 1 } else { mutable };

            if should_mutate {
                array.elements.borrow_mut()[index as usize] = value;
                Value::ArrayOrSlice(array.clone())
            } else {
                let mut elements = array.elements.borrow().to_vec();
                elements[index as usize] = value;
                let elements = Shared::new(elements);
                let rc = Shared::new(1);
                let element_types = array.element_types.clone();
                let is_slice = array.is_slice;
                Value::ArrayOrSlice(ArrayValue { elements, rc, element_types, is_slice })
            }
        } else {
            // Side effects are disabled, return the original array
            self.lookup(array)
        };
        self.define(result, result_array);
    }

    fn interpret_inc_rc(&self, array: ValueId) {
        if self.in_unconstrained_context() {
            let array = self.lookup(array);
            let array = array.as_array_or_slice().unwrap();
            let mut rc = array.rc.borrow_mut();

            assert_ne!(*rc, 0, "inc_rc: increment from 0 back to 1 detected");
            *rc += 1;
        }
    }

    fn interpret_dec_rc(&self, array: ValueId) {
        if self.in_unconstrained_context() {
            let array = self.lookup(array);
            let array = array.as_array_or_slice().unwrap();
            let mut rc = array.rc.borrow_mut();

            assert_ne!(*rc, 0, "dec_rc: underflow detected");
            *rc -= 1;
        }
    }

    fn interpret_if_else(
        &mut self,
        then_condition: ValueId,
        then_value: ValueId,
        else_condition: ValueId,
        else_value: ValueId,
        result: ValueId,
    ) {
        let then_condition = self.lookup(then_condition).as_bool().unwrap();
        let else_condition = self.lookup(else_condition).as_bool().unwrap();
        let then_value = self.lookup(then_value);
        let else_value = self.lookup(else_value);

        // Note that `then_condition = !else_condition` doesn't always hold!
        // Notably if this is a nested if expression we could have something like:
        //   then_condition = outer_condition & a
        //   else_condition = outer_condition & !a
        // If `outer_condition` is false, both will be false.
        assert!(!then_condition || !else_condition);

        let new_result = if then_condition { then_value } else { else_value };

        self.define(result, new_result);
    }

    fn interpret_make_array(
        &mut self,
        elements: &im::Vector<ValueId>,
        result: ValueId,
        result_type: &Type,
    ) {
        let elements = vecmap(elements, |element| self.lookup(*element));
        let is_slice = matches!(&result_type, Type::Slice(..));

        let array = Value::ArrayOrSlice(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types: result_type.clone().element_types(),
            is_slice,
        });
        self.define(result, array);
    }
}

macro_rules! apply_int_binop {
    ($lhs:expr, $rhs:expr, $f:expr) => {{
        use value::NumericValue::*;
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => panic!("Expected only integer values, found field values"),
            (U1(_), U1(_)) => panic!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U8($f(&lhs, &rhs)),
            (U16(lhs), U16(rhs)) => U16($f(&lhs, &rhs)),
            (U32(lhs), U32(rhs)) => U32($f(&lhs, &rhs)),
            (U64(lhs), U64(rhs)) => U64($f(&lhs, &rhs)),
            (U128(lhs), U128(rhs)) => U128($f(&lhs, &rhs)),
            (I8(lhs), I8(rhs)) => I8($f(&lhs, &rhs)),
            (I16(lhs), I16(rhs)) => I16($f(&lhs, &rhs)),
            (I32(lhs), I32(rhs)) => I32($f(&lhs, &rhs)),
            (I64(lhs), I64(rhs)) => I64($f(&lhs, &rhs)),
            (lhs, rhs) => panic!("Got mismatched types in binop: {lhs:?} and {rhs:?}"),
        }
    }};
}

macro_rules! apply_int_binop_opt {
    ($lhs:expr, $rhs:expr, $f:expr) => {{
        use value::NumericValue::*;
        // TODO: Error if None instead of unwrapping
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => panic!("Expected only integer values, found field values"),
            (U1(_), U1(_)) => panic!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U8($f(&lhs, &rhs).unwrap()),
            (U16(lhs), U16(rhs)) => U16($f(&lhs, &rhs).unwrap()),
            (U32(lhs), U32(rhs)) => U32($f(&lhs, &rhs).unwrap()),
            (U64(lhs), U64(rhs)) => U64($f(&lhs, &rhs).unwrap()),
            (U128(lhs), U128(rhs)) => U128($f(&lhs, &rhs).unwrap()),
            (I8(lhs), I8(rhs)) => I8($f(&lhs, &rhs).unwrap()),
            (I16(lhs), I16(rhs)) => I16($f(&lhs, &rhs).unwrap()),
            (I32(lhs), I32(rhs)) => I32($f(&lhs, &rhs).unwrap()),
            (I64(lhs), I64(rhs)) => I64($f(&lhs, &rhs).unwrap()),
            (lhs, rhs) => panic!("Got mismatched types in binop: {lhs:?} and {rhs:?}"),
        }
    }};
}

macro_rules! apply_int_comparison_op {
    ($lhs:expr, $rhs:expr, $f:expr) => {{
        use NumericValue::*;
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => panic!("Expected only integer values, found field values"),
            (U1(_), U1(_)) => panic!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U1($f(&lhs, &rhs)),
            (U16(lhs), U16(rhs)) => U1($f(&lhs, &rhs)),
            (U32(lhs), U32(rhs)) => U1($f(&lhs, &rhs)),
            (U64(lhs), U64(rhs)) => U1($f(&lhs, &rhs)),
            (U128(lhs), U128(rhs)) => U1($f(&lhs, &rhs)),
            (I8(lhs), I8(rhs)) => U1($f(&lhs, &rhs)),
            (I16(lhs), I16(rhs)) => U1($f(&lhs, &rhs)),
            (I32(lhs), I32(rhs)) => U1($f(&lhs, &rhs)),
            (I64(lhs), I64(rhs)) => U1($f(&lhs, &rhs)),
            (lhs, rhs) => panic!("Got mismatched types in binop: {lhs:?} and {rhs:?}"),
        }
    }};
}

impl Interpreter<'_> {
    fn interpret_binary(&mut self, binary: &Binary) -> IResult<Value> {
        // TODO: Replace unwrap with real error
        let lhs = self.lookup(binary.lhs).as_numeric().unwrap();
        let rhs = self.lookup(binary.rhs).as_numeric().unwrap();

        if lhs.get_type() != rhs.get_type()
            && !matches!(binary.operator, BinaryOp::Shl | BinaryOp::Shr)
        {
            panic!(
                "Type error in ({}: {}) {} ({}: {})",
                binary.lhs,
                lhs.get_type(),
                binary.operator,
                binary.rhs,
                rhs.get_type()
            )
        }

        // Disable this instruction if it is side-effectful and side effects are disabled.
        if !self.side_effects_enabled() && binary.requires_acir_gen_predicate(self.dfg()) {
            let zero = NumericValue::from_constant(FieldElement::zero(), lhs.get_type());
            return Ok(Value::Numeric(zero));
        }

        if let (Some(lhs), Some(rhs)) = (lhs.as_field(), rhs.as_field()) {
            return self.interpret_field_binary_op(lhs, binary.operator, rhs);
        }

        if let (Some(lhs), Some(rhs)) = (lhs.as_bool(), rhs.as_bool()) {
            return self.interpret_u1_binary_op(lhs, binary.operator, rhs);
        }

        let result = match binary.operator {
            BinaryOp::Add { unchecked: false } => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedAdd::checked_add)
            }
            BinaryOp::Add { unchecked: true } => {
                apply_int_binop!(lhs, rhs, num_traits::WrappingAdd::wrapping_add)
            }
            BinaryOp::Sub { unchecked: false } => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedSub::checked_sub)
            }
            BinaryOp::Sub { unchecked: true } => {
                apply_int_binop!(lhs, rhs, num_traits::WrappingSub::wrapping_sub)
            }
            BinaryOp::Mul { unchecked: false } => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedMul::checked_mul)
            }
            BinaryOp::Mul { unchecked: true } => {
                apply_int_binop!(lhs, rhs, num_traits::WrappingMul::wrapping_mul)
            }
            BinaryOp::Div => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedDiv::checked_div)
            }
            BinaryOp::Mod => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedRem::checked_rem)
            }
            BinaryOp::Eq => apply_int_comparison_op!(lhs, rhs, |a, b| a == b),
            BinaryOp::Lt => apply_int_comparison_op!(lhs, rhs, |a, b| a < b),
            BinaryOp::And => {
                apply_int_binop!(lhs, rhs, std::ops::BitAnd::bitand)
            }
            BinaryOp::Or => {
                apply_int_binop!(lhs, rhs, std::ops::BitOr::bitor)
            }
            BinaryOp::Xor => {
                apply_int_binop!(lhs, rhs, std::ops::BitXor::bitxor)
            }
            BinaryOp::Shl => {
                let rhs = rhs.as_u32().expect("Expected rhs of shl to be a u32");
                let overflow_msg = "Overflow when evaluating `shl`, `rhs` is too large";
                use NumericValue::*;
                match lhs {
                    Field(_) => unreachable!("<< is not implemented for Field"),
                    U1(_) => unreachable!("<< is not implemented for u1"),
                    U8(value) => U8(value.checked_shl(rhs).expect(overflow_msg)),
                    U16(value) => U16(value.checked_shl(rhs).expect(overflow_msg)),
                    U32(value) => U32(value.checked_shl(rhs).expect(overflow_msg)),
                    U64(value) => U64(value.checked_shl(rhs).expect(overflow_msg)),
                    U128(value) => U128(value.checked_shl(rhs).expect(overflow_msg)),
                    I8(value) => I8(value.checked_shl(rhs).expect(overflow_msg)),
                    I16(value) => I16(value.checked_shl(rhs).expect(overflow_msg)),
                    I32(value) => I32(value.checked_shl(rhs).expect(overflow_msg)),
                    I64(value) => I64(value.checked_shl(rhs).expect(overflow_msg)),
                }
            }
            BinaryOp::Shr => {
                let rhs = rhs.as_u32().expect("Expected rhs of shr to be a u32");
                let overflow_msg = "Overflow when evaluating `shr`, `rhs` is too large";
                use NumericValue::*;
                match lhs {
                    Field(_) => unreachable!(">> is not implemented for Field"),
                    U1(_) => unreachable!(">> is not implemented for u1"),
                    U8(value) => U8(value.checked_shr(rhs).expect(overflow_msg)),
                    U16(value) => U16(value.checked_shr(rhs).expect(overflow_msg)),
                    U32(value) => U32(value.checked_shr(rhs).expect(overflow_msg)),
                    U64(value) => U64(value.checked_shr(rhs).expect(overflow_msg)),
                    U128(value) => U128(value.checked_shr(rhs).expect(overflow_msg)),
                    I8(value) => I8(value.checked_shr(rhs).expect(overflow_msg)),
                    I16(value) => I16(value.checked_shr(rhs).expect(overflow_msg)),
                    I32(value) => I32(value.checked_shr(rhs).expect(overflow_msg)),
                    I64(value) => I64(value.checked_shr(rhs).expect(overflow_msg)),
                }
            }
        };
        Ok(Value::Numeric(result))
    }

    fn interpret_field_binary_op(
        &mut self,
        lhs: FieldElement,
        operator: BinaryOp,
        rhs: FieldElement,
    ) -> IResult<Value> {
        let result = match operator {
            BinaryOp::Add { unchecked: _ } => NumericValue::Field(lhs + rhs),
            BinaryOp::Sub { unchecked: _ } => NumericValue::Field(lhs - rhs),
            BinaryOp::Mul { unchecked: _ } => NumericValue::Field(lhs * rhs),
            BinaryOp::Div => {
                // FieldElement::div returns a value with panicking on divide by zero
                if rhs.is_zero() {
                    panic!("Field division by zero");
                }
                NumericValue::Field(lhs / rhs)
            }
            BinaryOp::Mod => panic!("Unsupported operator `%` for Field"),
            BinaryOp::Eq => NumericValue::U1(lhs == rhs),
            BinaryOp::Lt => NumericValue::U1(lhs < rhs),
            BinaryOp::And => panic!("Unsupported operator `&` for Field"),
            BinaryOp::Or => panic!("Unsupported operator `|` for Field"),
            BinaryOp::Xor => panic!("Unsupported operator `^` for Field"),
            BinaryOp::Shl => panic!("Unsupported operator `<<` for Field"),
            BinaryOp::Shr => panic!("Unsupported operator `>>` for Field"),
        };
        Ok(Value::Numeric(result))
    }

    fn interpret_u1_binary_op(
        &mut self,
        lhs: bool,
        operator: BinaryOp,
        rhs: bool,
    ) -> IResult<Value> {
        let result = match operator {
            BinaryOp::Add { unchecked: _ } => panic!("Unsupported operator `+` for u1"),
            BinaryOp::Sub { unchecked: _ } => panic!("Unsupported operator `-` for u1"),
            BinaryOp::Mul { unchecked: _ } => lhs & rhs, // (*) = (&) for u1
            BinaryOp::Div => panic!("Unsupported operator `/` for u1"),
            BinaryOp::Mod => panic!("Unsupported operator `%` for u1"),
            BinaryOp::Eq => lhs == rhs,
            // clippy complains when you do `lhs < rhs` and recommends this instead
            BinaryOp::Lt => !lhs & rhs,
            BinaryOp::And => lhs & rhs,
            BinaryOp::Or => lhs | rhs,
            BinaryOp::Xor => lhs ^ rhs,
            BinaryOp::Shl => panic!("Unsupported operator `<<` for u1"),
            BinaryOp::Shr => panic!("Unsupported operator `>>` for u1"),
        };
        Ok(Value::Numeric(NumericValue::U1(result)))
    }
}

fn truncate_unsigned<T>(value: T, bit_size: u32) -> T
where
    u128: From<T>,
    T: TryFrom<u128>,
    <T as TryFrom<u128>>::Error: std::fmt::Debug,
{
    let value_u128 = u128::from(value);
    let bit_mask = match bit_size.cmp(&128) {
        Ordering::Less => (1u128 << bit_size) - 1,
        Ordering::Equal => u128::MAX,
        Ordering::Greater => panic!("truncate: Invalid bit size: {bit_size}"),
    };

    let result = value_u128 & bit_mask;
    T::try_from(result).expect(
        "The truncated result should always be smaller than or equal to the original `value`",
    )
}

fn truncate_signed<T>(value: T, bit_size: u32) -> T
where
    i128: From<T>,
    T: TryFrom<i128> + num_traits::Bounded,
    <T as TryFrom<i128>>::Error: std::fmt::Debug,
{
    let mut value_i128 = i128::from(value);
    if value_i128 < 0 {
        let max = 1i128 << (bit_size - 1);
        value_i128 += max;
        assert!(bit_size <= 64, "The maximum bit size for signed integers is 64");

        let mask = (1i128 << bit_size) - 1;
        let result = (value_i128 & mask) - max;

        T::try_from(result).expect(
            "The truncated result should always be smaller than or equal to the original `value`",
        )
    } else {
        let result = truncate_unsigned::<u128>(value_i128 as u128, bit_size) as i128;
        T::try_from(result).expect(
            "The truncated result should always be smaller than or equal to the original `value`",
        )
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_truncate_unsigned() {
        assert_eq!(super::truncate_unsigned(57_u32, 8), 57);
        assert_eq!(super::truncate_unsigned(257_u16, 8), 1);
        assert_eq!(super::truncate_unsigned(130_u8, 7), 2);
        assert_eq!(super::truncate_unsigned(u8::MAX, 8), u8::MAX);
        assert_eq!(super::truncate_unsigned(u128::MAX, 128), u128::MAX);
    }

    #[test]
    fn test_truncate_signed() {
        assert_eq!(super::truncate_signed(57_i32, 8), 57);
        assert_eq!(super::truncate_signed(257_i16, 8), 1);
        assert_eq!(super::truncate_signed(130_i64, 7), 2);
        assert_eq!(super::truncate_signed(i16::MAX, 16), i16::MAX);

        assert_eq!(super::truncate_signed(-57_i32, 8), -57);
        assert_eq!(super::truncate_signed(-1_i64, 3), -1_i64);
        assert_eq!(super::truncate_signed(-258_i16, 8), -2);
        assert_eq!(super::truncate_signed(-130_i16, 7), -2);
        assert_eq!(super::truncate_signed(i8::MIN, 8), i8::MIN);
        assert_eq!(super::truncate_signed(-8_i8, 4), -8);
        assert_eq!(super::truncate_signed(-8_i8, 3), 0);
        assert_eq!(super::truncate_signed(-129_i32, 8), 127);
    }
}
