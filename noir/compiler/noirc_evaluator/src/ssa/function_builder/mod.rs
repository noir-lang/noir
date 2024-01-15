pub(crate) mod data_bus;

use std::{borrow::Cow, rc::Rc};

use acvm::FieldElement;
use noirc_errors::Location;

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    function::{Function, FunctionId},
    instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
    types::Type,
    value::{Value, ValueId},
};

use super::{
    ir::{
        basic_block::BasicBlock,
        dfg::{CallStack, InsertInstructionResult},
        function::RuntimeType,
        instruction::{Endian, InstructionId, Intrinsic},
        types::NumericType,
    },
    ssa_gen::Ssa,
};

/// The per-function context for each ssa function being generated.
///
/// This is split from the global SsaBuilder context to allow each function
/// to be potentially built concurrently.
///
/// Contrary to the name, this struct has the capacity to build as many
/// functions as needed, although it is limited to one function at a time.
pub(crate) struct FunctionBuilder {
    pub(super) current_function: Function,
    current_block: BasicBlockId,
    finished_functions: Vec<Function>,
    call_stack: CallStack,
}

impl FunctionBuilder {
    /// Creates a new FunctionBuilder to build the function with the given FunctionId.
    ///
    /// This creates the new function internally so there is no need to call .new_function()
    /// right after constructing a new FunctionBuilder.
    pub(crate) fn new(
        function_name: String,
        function_id: FunctionId,
        runtime: RuntimeType,
    ) -> Self {
        let mut new_function = Function::new(function_name, function_id);
        new_function.set_runtime(runtime);
        let current_block = new_function.entry_block();

        Self {
            current_function: new_function,
            current_block,
            finished_functions: Vec::new(),
            call_stack: CallStack::new(),
        }
    }

    /// Finish the current function and create a new function.
    ///
    /// A FunctionBuilder can always only work on one function at a time, so care
    /// should be taken not to finish a function that is still in progress by calling
    /// new_function before the current function is finished.
    fn new_function_with_type(
        &mut self,
        name: String,
        function_id: FunctionId,
        runtime_type: RuntimeType,
    ) {
        let mut new_function = Function::new(name, function_id);
        new_function.set_runtime(runtime_type);
        self.current_block = new_function.entry_block();

        let old_function = std::mem::replace(&mut self.current_function, new_function);
        self.finished_functions.push(old_function);
    }

    /// Finish the current function and create a new ACIR function.
    pub(crate) fn new_function(&mut self, name: String, function_id: FunctionId) {
        self.new_function_with_type(name, function_id, RuntimeType::Acir);
    }

    /// Finish the current function and create a new unconstrained function.
    pub(crate) fn new_brillig_function(&mut self, name: String, function_id: FunctionId) {
        self.new_function_with_type(name, function_id, RuntimeType::Brillig);
    }

    /// Consume the FunctionBuilder returning all the functions it has generated.
    pub(crate) fn finish(mut self) -> Ssa {
        self.finished_functions.push(self.current_function);
        Ssa::new(self.finished_functions)
    }

    /// Add a parameter to the current function with the given parameter type.
    /// Returns the newly-added parameter.
    pub(crate) fn add_parameter(&mut self, typ: Type) -> ValueId {
        let entry = self.current_function.entry_block();
        self.current_function.dfg.add_block_parameter(entry, typ)
    }

    /// Insert a numeric constant into the current function
    pub(crate) fn numeric_constant(
        &mut self,
        value: impl Into<FieldElement>,
        typ: Type,
    ) -> ValueId {
        self.current_function.dfg.make_constant(value.into(), typ)
    }

    /// Insert a numeric constant into the current function of type Field
    pub(crate) fn field_constant(&mut self, value: impl Into<FieldElement>) -> ValueId {
        self.numeric_constant(value.into(), Type::field())
    }

    /// Insert an array constant into the current function with the given element values.
    pub(crate) fn array_constant(&mut self, elements: im::Vector<ValueId>, typ: Type) -> ValueId {
        self.current_function.dfg.make_array(elements, typ)
    }

    /// Returns the type of the given value.
    pub(crate) fn type_of_value(&self, value: ValueId) -> Type {
        self.current_function.dfg.type_of_value(value)
    }

    /// Insert a new block into the current function and return it.
    /// Note that this block is unreachable until another block is set to jump to it.
    pub(crate) fn insert_block(&mut self) -> BasicBlockId {
        self.current_function.dfg.make_block()
    }

    /// Adds a parameter with the given type to the given block.
    /// Returns the newly-added parameter.
    pub(crate) fn add_block_parameter(&mut self, block: BasicBlockId, typ: Type) -> ValueId {
        self.current_function.dfg.add_block_parameter(block, typ)
    }

    /// Returns the parameters of the given block in the current function.
    pub(crate) fn block_parameters(&self, block: BasicBlockId) -> &[ValueId] {
        self.current_function.dfg.block_parameters(block)
    }

    /// Inserts a new instruction at the end of the current block and returns its results
    pub(crate) fn insert_instruction(
        &mut self,
        instruction: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> InsertInstructionResult {
        self.current_function.dfg.insert_instruction_and_results(
            instruction,
            self.current_block,
            ctrl_typevars,
            self.call_stack.clone(),
        )
    }

    /// Switch to inserting instructions in the given block.
    /// Expects the given block to be within the same function. If you want to insert
    /// instructions into a new function, call new_function instead.
    pub(crate) fn switch_to_block(&mut self, block: BasicBlockId) {
        self.current_block = block;
    }

    /// Returns the block currently being inserted into
    pub(crate) fn current_block(&mut self) -> BasicBlockId {
        self.current_block
    }

    /// Insert an allocate instruction at the end of the current block, allocating the
    /// given amount of field elements. Returns the result of the allocate instruction,
    /// which is always a Reference to the allocated data.
    pub(crate) fn insert_allocate(&mut self, element_type: Type) -> ValueId {
        let reference_type = Type::Reference(Rc::new(element_type));
        self.insert_instruction(Instruction::Allocate, Some(vec![reference_type])).first()
    }

    pub(crate) fn set_location(&mut self, location: Location) -> &mut FunctionBuilder {
        self.call_stack = im::Vector::unit(location);
        self
    }

    pub(crate) fn set_call_stack(&mut self, call_stack: CallStack) -> &mut FunctionBuilder {
        self.call_stack = call_stack;
        self
    }

    pub(crate) fn get_call_stack(&self) -> CallStack {
        self.call_stack.clone()
    }

    /// Insert a Load instruction at the end of the current block, loading from the given offset
    /// of the given address which should point to a previous Allocate instruction. Note that
    /// this is limited to loading a single value. Loading multiple values (such as a tuple)
    /// will require multiple loads.
    /// 'offset' is in units of FieldElements here. So loading the fourth FieldElement stored in
    /// an array will have an offset of 3.
    /// Returns the element that was loaded.
    pub(crate) fn insert_load(&mut self, address: ValueId, type_to_load: Type) -> ValueId {
        self.insert_instruction(Instruction::Load { address }, Some(vec![type_to_load])).first()
    }

    /// Insert a Store instruction at the end of the current block, storing the given element
    /// at the given address. Expects that the address points somewhere
    /// within a previous Allocate instruction.
    pub(crate) fn insert_store(&mut self, address: ValueId, value: ValueId) {
        self.insert_instruction(Instruction::Store { address, value }, None);
    }

    /// Insert a binary instruction at the end of the current block.
    /// Returns the result of the binary instruction.
    pub(crate) fn insert_binary(
        &mut self,
        lhs: ValueId,
        operator: BinaryOp,
        rhs: ValueId,
    ) -> ValueId {
        let instruction = Instruction::Binary(Binary { lhs, rhs, operator });
        self.insert_instruction(instruction, None).first()
    }

    /// Insert a not instruction at the end of the current block.
    /// Returns the result of the instruction.
    pub(crate) fn insert_not(&mut self, rhs: ValueId) -> ValueId {
        self.insert_instruction(Instruction::Not(rhs), None).first()
    }

    /// Insert a cast instruction at the end of the current block.
    /// Returns the result of the cast instruction.
    pub(crate) fn insert_cast(&mut self, value: ValueId, typ: Type) -> ValueId {
        self.insert_instruction(Instruction::Cast(value, typ), None).first()
    }

    /// Insert a truncate instruction at the end of the current block.
    /// Returns the result of the truncate instruction.
    pub(crate) fn insert_truncate(
        &mut self,
        value: ValueId,
        bit_size: u32,
        max_bit_size: u32,
    ) -> ValueId {
        self.insert_instruction(Instruction::Truncate { value, bit_size, max_bit_size }, None)
            .first()
    }

    /// Insert a constrain instruction at the end of the current block.
    pub(crate) fn insert_constrain(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        assert_message: Option<String>,
    ) {
        self.insert_instruction(Instruction::Constrain(lhs, rhs, assert_message), None);
    }

    /// Insert a [`Instruction::RangeCheck`] instruction at the end of the current block.
    pub(crate) fn insert_range_check(
        &mut self,
        value: ValueId,
        max_bit_size: u32,
        assert_message: Option<String>,
    ) {
        self.insert_instruction(
            Instruction::RangeCheck { value, max_bit_size, assert_message },
            None,
        );
    }

    /// Insert a call instruction at the end of the current block and return
    /// the results of the call.
    pub(crate) fn insert_call(
        &mut self,
        func: ValueId,
        arguments: Vec<ValueId>,
        result_types: Vec<Type>,
    ) -> Cow<[ValueId]> {
        self.insert_instruction(Instruction::Call { func, arguments }, Some(result_types)).results()
    }

    /// Insert ssa instructions which computes lhs << rhs by doing lhs*2^rhs
    /// and truncate the result to bit_size
    pub(crate) fn insert_wrapping_shift_left(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        bit_size: u32,
    ) -> ValueId {
        let base = self.field_constant(FieldElement::from(2_u128));
        let typ = self.current_function.dfg.type_of_value(lhs);
        let (max_bit, pow) =
            if let Some(rhs_constant) = self.current_function.dfg.get_numeric_constant(rhs) {
                // Happy case is that we know precisely by how many bits the the integer will
                // increase: lhs_bit_size + rhs
                let bit_shift_size = rhs_constant.to_u128() as u32;

                let (rhs_bit_size_pow_2, overflows) = 2_u128.overflowing_pow(bit_shift_size);
                if overflows {
                    assert!(bit_size < 128, "ICE - shift left with big integers are not supported");
                    if bit_size < 128 {
                        let zero = self.numeric_constant(FieldElement::zero(), typ);
                        return InsertInstructionResult::SimplifiedTo(zero).first();
                    }
                }
                let pow = self.numeric_constant(FieldElement::from(rhs_bit_size_pow_2), typ);

                let max_lhs_bits = self.current_function.dfg.get_value_max_num_bits(lhs);

                (max_lhs_bits + bit_shift_size, pow)
            } else {
                // we use a predicate to nullify the result in case of overflow
                let bit_size_var =
                    self.numeric_constant(FieldElement::from(bit_size as u128), typ.clone());
                let overflow = self.insert_binary(rhs, BinaryOp::Lt, bit_size_var);
                let one = self.numeric_constant(FieldElement::one(), Type::unsigned(1));
                let predicate = self.insert_binary(overflow, BinaryOp::Eq, one);
                let predicate = self.insert_cast(predicate, typ.clone());
                // we can safely cast to unsigned because overflow_checks prevent bit-shift with a negative value
                let rhs_unsigned = self.insert_cast(rhs, Type::unsigned(bit_size));
                let pow = self.pow(base, rhs_unsigned);
                let pow = self.insert_cast(pow, typ);
                (FieldElement::max_num_bits(), self.insert_binary(predicate, BinaryOp::Mul, pow))
            };

        if max_bit <= bit_size {
            self.insert_binary(lhs, BinaryOp::Mul, pow)
        } else {
            let result = self.insert_binary(lhs, BinaryOp::Mul, pow);
            self.insert_truncate(result, bit_size, max_bit)
        }
    }

    /// Insert ssa instructions which computes lhs >> rhs by doing lhs/2^rhs
    pub(crate) fn insert_shift_right(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        bit_size: u32,
    ) -> ValueId {
        let base = self.field_constant(FieldElement::from(2_u128));
        // we can safely cast to unsigned because overflow_checks prevent bit-shift with a negative value
        let rhs_unsigned = self.insert_cast(rhs, Type::unsigned(bit_size));
        let pow = self.pow(base, rhs_unsigned);
        self.insert_binary(lhs, BinaryOp::Div, pow)
    }

    /// Computes lhs^rhs via square&multiply, using the bits decomposition of rhs
    /// Pseudo-code of the computation:
    /// let mut r = 1;
    /// let rhs_bits = to_bits(rhs);
    /// for i in 1 .. bit_size + 1 {
    ///     let r_squared = r * r;
    ///     let b = rhs_bits[bit_size - i];
    ///     r = (r_squared * lhs * b) + (1 - b) * r_squared;
    /// }
    pub(crate) fn pow(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let typ = self.current_function.dfg.type_of_value(rhs);
        if let Type::Numeric(NumericType::Unsigned { bit_size }) = typ {
            let to_bits = self.import_intrinsic_id(Intrinsic::ToBits(Endian::Little));
            let length = self.field_constant(FieldElement::from(bit_size as i128));
            let result_types =
                vec![Type::field(), Type::Array(Rc::new(vec![Type::bool()]), bit_size as usize)];
            let rhs_bits = self.insert_call(to_bits, vec![rhs, length], result_types);
            let rhs_bits = rhs_bits[1];
            let one = self.field_constant(FieldElement::one());
            let mut r = one;
            for i in 1..bit_size + 1 {
                let r_squared = self.insert_binary(r, BinaryOp::Mul, r);
                let a = self.insert_binary(r_squared, BinaryOp::Mul, lhs);
                let idx = self.field_constant(FieldElement::from((bit_size - i) as i128));
                let b = self.insert_array_get(rhs_bits, idx, Type::field());
                let r1 = self.insert_binary(a, BinaryOp::Mul, b);
                let c = self.insert_binary(one, BinaryOp::Sub, b);
                let r2 = self.insert_binary(c, BinaryOp::Mul, r_squared);
                r = self.insert_binary(r1, BinaryOp::Add, r2);
            }
            r
        } else {
            unreachable!("Value must be unsigned in power operation");
        }
    }

    /// Insert an instruction to extract an element from an array
    pub(crate) fn insert_array_get(
        &mut self,
        array: ValueId,
        index: ValueId,
        element_type: Type,
    ) -> ValueId {
        let element_type = Some(vec![element_type]);
        self.insert_instruction(Instruction::ArrayGet { array, index }, element_type).first()
    }

    /// Insert an instruction to create a new array with the given index replaced with a new value
    pub(crate) fn insert_array_set(
        &mut self,
        array: ValueId,
        index: ValueId,
        value: ValueId,
    ) -> ValueId {
        self.insert_instruction(Instruction::ArraySet { array, index, value }, None).first()
    }

    /// Terminates the current block with the given terminator instruction
    fn terminate_block_with(&mut self, terminator: TerminatorInstruction) {
        self.current_function.dfg.set_block_terminator(self.current_block, terminator);
    }

    /// Terminate the current block with a jmp instruction to jmp to the given
    /// block with the given arguments.
    pub(crate) fn terminate_with_jmp(
        &mut self,
        destination: BasicBlockId,
        arguments: Vec<ValueId>,
    ) {
        let call_stack = self.call_stack.clone();
        self.terminate_block_with(TerminatorInstruction::Jmp {
            destination,
            arguments,
            call_stack,
        });
    }

    /// Terminate the current block with a jmpif instruction to jmp with the given arguments
    /// block with the given arguments.
    pub(crate) fn terminate_with_jmpif(
        &mut self,
        condition: ValueId,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
    ) {
        self.terminate_block_with(TerminatorInstruction::JmpIf {
            condition,
            then_destination,
            else_destination,
        });
    }

    /// Terminate the current block with a return instruction
    pub(crate) fn terminate_with_return(&mut self, return_values: Vec<ValueId>) {
        let call_stack = self.call_stack.clone();
        self.terminate_block_with(TerminatorInstruction::Return { return_values, call_stack });
    }

    /// Returns a ValueId pointing to the given function or imports the function
    /// into the current function if it was not already, and returns that ID.
    pub(crate) fn import_function(&mut self, function: FunctionId) -> ValueId {
        self.current_function.dfg.import_function(function)
    }

    /// Returns a ValueId pointing to the given oracle/foreign function or imports the oracle
    /// into the current function if it was not already, and returns that ID.
    pub(crate) fn import_foreign_function(&mut self, function: &str) -> ValueId {
        self.current_function.dfg.import_foreign_function(function)
    }

    /// Retrieve a value reference to the given intrinsic operation.
    /// Returns None if there is no intrinsic matching the given name.
    pub(crate) fn import_intrinsic(&mut self, name: &str) -> Option<ValueId> {
        Intrinsic::lookup(name).map(|intrinsic| self.import_intrinsic_id(intrinsic))
    }

    /// Retrieve a value reference to the given intrinsic operation.
    pub(crate) fn import_intrinsic_id(&mut self, intrinsic: Intrinsic) -> ValueId {
        self.current_function.dfg.import_intrinsic(intrinsic)
    }

    pub(crate) fn get_intrinsic_from_value(&mut self, value: ValueId) -> Option<Intrinsic> {
        match self.current_function.dfg[value] {
            Value::Intrinsic(intrinsic) => Some(intrinsic),
            _ => None,
        }
    }

    /// Insert instructions to increment the reference count of any array(s) stored
    /// within the given value. If the given value is not an array and does not contain
    /// any arrays, this does nothing.
    pub(crate) fn increment_array_reference_count(&mut self, value: ValueId) {
        match self.type_of_value(value) {
            Type::Numeric(_) => (),
            Type::Function => (),
            Type::Reference(element) => {
                if element.contains_an_array() {
                    let value = self.insert_load(value, element.as_ref().clone());
                    self.increment_array_reference_count(value);
                }
            }
            Type::Array(..) | Type::Slice(..) => {
                self.insert_instruction(Instruction::IncrementRc { value }, None);
                // If there are nested arrays or slices, we wait until ArrayGet
                // is issued to increment the count of that array.
            }
        }
    }
}

impl std::ops::Index<ValueId> for FunctionBuilder {
    type Output = Value;

    fn index(&self, id: ValueId) -> &Self::Output {
        &self.current_function.dfg[id]
    }
}

impl std::ops::Index<InstructionId> for FunctionBuilder {
    type Output = Instruction;

    fn index(&self, id: InstructionId) -> &Self::Output {
        &self.current_function.dfg[id]
    }
}

impl std::ops::Index<BasicBlockId> for FunctionBuilder {
    type Output = BasicBlock;

    fn index(&self, id: BasicBlockId) -> &Self::Output {
        &self.current_function.dfg[id]
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use acvm::FieldElement;

    use crate::ssa::ir::{
        function::RuntimeType,
        instruction::{Endian, Intrinsic},
        map::Id,
        types::Type,
        value::Value,
    };

    use super::FunctionBuilder;

    #[test]
    fn insert_constant_call() {
        // `bits` should be an array of constants [1, 1, 1, 0...] of length 8:
        // let x = 7;
        // let bits = x.to_le_bits(8);
        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id, RuntimeType::Acir);
        let one = builder.numeric_constant(FieldElement::one(), Type::bool());
        let zero = builder.numeric_constant(FieldElement::zero(), Type::bool());

        let to_bits_id = builder.import_intrinsic_id(Intrinsic::ToBits(Endian::Little));
        let input = builder.numeric_constant(FieldElement::from(7_u128), Type::field());
        let length = builder.numeric_constant(FieldElement::from(8_u128), Type::field());
        let result_types = vec![Type::Array(Rc::new(vec![Type::bool()]), 8)];
        let call_results =
            builder.insert_call(to_bits_id, vec![input, length], result_types).into_owned();

        let slice_len = match &builder.current_function.dfg[call_results[0]] {
            Value::NumericConstant { constant, .. } => *constant,
            _ => panic!(),
        };
        assert_eq!(slice_len, FieldElement::from(8_u128));

        let slice = match &builder.current_function.dfg[call_results[1]] {
            Value::Array { array, .. } => array,
            _ => panic!(),
        };
        assert_eq!(slice[0], one);
        assert_eq!(slice[1], one);
        assert_eq!(slice[2], one);
        assert_eq!(slice[3], zero);
    }
}
