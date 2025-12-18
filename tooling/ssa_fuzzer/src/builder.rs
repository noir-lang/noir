use crate::typed_value::{NumericType, Point, Scalar, Type, TypedValue};
use acvm::FieldElement;
use noir_ssa_executor::compiler::compile_from_ssa;
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_evaluator::ssa::function_builder::FunctionBuilder;
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use noirc_evaluator::ssa::ir::function::{Function, RuntimeType};
use noirc_evaluator::ssa::ir::instruction::BinaryOp;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::types::Type as SsaType;
use noirc_evaluator::ssa::ir::value::Value;
use noirc_frontend::monomorphization::ast::InlineType as FrontendInlineType;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FuzzerBuilderError {
    #[error("Compilation panicked: {0}")]
    RuntimeError(String),
}

/// Inserts binary instruction with two arguments and returns Id of the result
/// eg add, sub, mul
pub type InstructionWithTwoArgs = fn(&mut FuzzerBuilder, TypedValue, TypedValue) -> TypedValue;

/// Inserts unary instruction with one argument and returns Id of the result
/// eg not, SimpleCast(casts variable to current numeric type)
pub type InstructionWithOneArg = fn(&mut FuzzerBuilder, TypedValue) -> TypedValue;

/// Builder for generating fuzzed SSA functions
/// Contains a FunctionBuilder and tracks the current numeric type being used
pub struct FuzzerBuilder {
    pub(crate) builder: FunctionBuilder,
    pub(crate) runtime: RuntimeType,
}

impl FuzzerBuilder {
    /// Creates a new FuzzerBuilder in ACIR context
    pub fn new_acir(simplifying_enabled: bool) -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Acir(FrontendInlineType::default()));
        builder.simplify = simplifying_enabled;
        Self { builder, runtime: RuntimeType::Acir(FrontendInlineType::default()) }
    }

    /// Creates a new FuzzerBuilder in Brillig context
    pub fn new_brillig(simplifying_enabled: bool) -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(FrontendInlineType::default()));
        builder.simplify = simplifying_enabled;
        Self { builder, runtime: RuntimeType::Brillig(FrontendInlineType::default()) }
    }

    pub fn new_by_runtime(runtime: RuntimeType, simplifying_enabled: bool) -> Self {
        let main_id: Id<Function> = Id::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(runtime);
        builder.simplify = simplifying_enabled;
        Self { builder, runtime }
    }

    /// Compiles the built function into a CompiledProgram, to run it with nargo execute
    pub fn compile(
        self,
        compile_options: CompileOptions,
    ) -> Result<CompiledProgram, FuzzerBuilderError> {
        let ssa = self.builder.finish();
        let result =
            std::panic::catch_unwind(AssertUnwindSafe(|| compile_from_ssa(ssa, &compile_options)));
        match result {
            Ok(result) => match result {
                Ok(result) => Ok(result),
                Err(e) => Err(FuzzerBuilderError::RuntimeError(format!("Compilation error {e:?}"))),
            },
            Err(_) => Err(FuzzerBuilderError::RuntimeError("Compilation panicked".to_string())),
        }
    }

    /// Inserts initial variables of the given type into the function
    pub fn insert_variable(&mut self, variable_type: SsaType) -> TypedValue {
        let id = self.builder.add_parameter(variable_type.clone());
        TypedValue::new(id, variable_type.into())
    }

    /// Terminates main function block with the given value
    pub fn finalize_function(&mut self, return_value: &TypedValue) {
        self.builder.terminate_with_return(vec![return_value.value_id]);
    }

    /// Inserts an add instruction between two values
    pub fn insert_add_instruction_checked(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(
            lhs.value_id,
            BinaryOp::Add { unchecked: false },
            rhs.value_id,
        );
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a subtract instruction between two values
    pub fn insert_sub_instruction_checked(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(
            lhs.value_id,
            BinaryOp::Sub { unchecked: false },
            rhs.value_id,
        );
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a multiply instruction between two values
    pub fn insert_mul_instruction_checked(
        &mut self,
        lhs: TypedValue,
        rhs: TypedValue,
    ) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(
            lhs.value_id,
            BinaryOp::Mul { unchecked: false },
            rhs.value_id,
        );
        let init_bit_length = lhs.bit_length();

        self.builder.insert_range_check(
            res,
            init_bit_length,
            Some("Attempt to multiply with overflow".to_string()),
        );
        // if field, no need to truncate
        if lhs.is_field() {
            return TypedValue::new(res, lhs.type_of_variable);
        }
        let bit_size = lhs.bit_length();
        let res = self.builder.insert_truncate(res, bit_size, bit_size * 2);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a divide instruction between two values
    pub fn insert_div_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Div, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a modulo instruction between two values
    pub fn insert_mod_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        assert!(!lhs.is_field(), "must not be field");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Mod, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a not instruction for the given value
    pub fn insert_not_instruction(&mut self, lhs: TypedValue) -> TypedValue {
        assert!(lhs.is_numeric(), "must be numeric");
        assert!(!lhs.is_field(), "must not be field");
        let res = self.builder.insert_not(lhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a cast instruction
    pub fn insert_cast(&mut self, value: TypedValue, cast_type: Type) -> TypedValue {
        assert!(value.is_numeric(), "must be numeric");
        assert!(cast_type.is_numeric(), "must be numeric, got {cast_type:?}");

        let init_bit_length = value.bit_length();

        let mut value_id = value.value_id;
        // if not field, truncate
        if cast_type.bit_length() != 254 {
            value_id = self.builder.insert_truncate(
                value.value_id,
                cast_type.bit_length(),
                init_bit_length,
            );
        }
        let cast_type_as_numeric_type = cast_type.unwrap_numeric();

        let res = self.builder.insert_cast(value_id, cast_type_as_numeric_type.into());
        TypedValue::new(res, cast_type)
    }

    /// Inserts an equals comparison instruction between two values
    pub fn insert_eq_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Eq, rhs.value_id);
        TypedValue::new(res, Type::Numeric(NumericType::Boolean))
    }

    /// Inserts a less than comparison instruction between two values
    pub fn insert_lt_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Lt, rhs.value_id);
        TypedValue::new(res, Type::Numeric(NumericType::Boolean))
    }

    /// Inserts a bitwise AND instruction between two values
    pub fn insert_and_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::And, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a bitwise OR instruction between two values
    pub fn insert_or_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Or, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a bitwise XOR instruction between two values
    pub fn insert_xor_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Xor, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a left shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shl_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Shl, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    /// Inserts a right shift instruction between two values
    /// The right hand side is cast to 8 bits
    pub fn insert_shr_instruction(&mut self, lhs: TypedValue, rhs: TypedValue) -> TypedValue {
        assert!(lhs.same_types(&rhs), "must have same type");
        assert!(lhs.is_numeric(), "must be numeric");
        let res = self.builder.insert_binary(lhs.value_id, BinaryOp::Shr, rhs.value_id);
        TypedValue::new(res, lhs.type_of_variable)
    }

    pub fn insert_constant(
        &mut self,
        value: impl Into<FieldElement>,
        type_: NumericType,
    ) -> TypedValue {
        let id = self.builder.numeric_constant(value.into(), type_.into());
        TypedValue::new(id, Type::Numeric(type_))
    }

    pub fn insert_constrain(&mut self, lhs: TypedValue, rhs: TypedValue) {
        self.builder.insert_constrain(lhs.value_id, rhs.value_id, None);
    }

    /// Gets the index of the entry block
    pub fn get_current_block(&self) -> BasicBlockId {
        self.builder.get_current_block_index()
    }

    /// Switches the current block to the given block
    pub fn switch_to_block(&mut self, block: BasicBlockId) {
        self.builder.switch_to_block(block);
    }

    /// Inserts a new basic block and returns its index
    pub fn insert_block(&mut self) -> BasicBlockId {
        self.builder.insert_block()
    }

    /// Inserts a new parameter to the current SSA block and returns its value
    ///
    /// b0() -> b0(new_parameter: parameter_type)
    pub fn add_block_parameter(&mut self, block: BasicBlockId, typ: Type) -> TypedValue {
        let id = self.builder.add_block_parameter(block, typ.clone().into());
        TypedValue::new(id, typ)
    }

    /// Inserts a return instruction with the given value
    pub fn insert_return_instruction(&mut self, return_value: Id<Value>) {
        self.builder.terminate_with_return(vec![return_value]);
    }

    /// Inserts an unconditional jump to the given block with parameters
    pub fn insert_jmp_instruction(
        &mut self,
        destination: BasicBlockId,
        arguments: Vec<TypedValue>,
    ) {
        self.builder.terminate_with_jmp(
            destination,
            arguments.into_iter().map(|arg| arg.value_id).collect(),
        );
    }

    /// Inserts a conditional jump based on the condition value
    pub fn insert_jmpif_instruction(
        &mut self,
        condition: Id<Value>,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
    ) {
        self.builder.terminate_with_jmpif(condition, then_destination, else_destination);
    }

    pub fn insert_add_to_memory(&mut self, lhs: TypedValue) -> TypedValue {
        let memory_address = self.builder.insert_allocate(lhs.clone().type_of_variable.into());
        self.builder.insert_store(memory_address, lhs.value_id);
        TypedValue::new(memory_address, Type::Reference(Arc::new(lhs.type_of_variable)))
    }

    pub fn insert_load_from_memory(&mut self, memory_addr: TypedValue) -> TypedValue {
        assert!(memory_addr.type_of_variable.is_reference(), "memory address must be a reference");
        let res = self.builder.insert_load(
            memory_addr.value_id,
            memory_addr.type_of_variable.unwrap_reference().into(),
        );
        TypedValue::new(res, memory_addr.type_of_variable.unwrap_reference())
    }

    pub fn insert_set_to_memory(&mut self, memory_addr: TypedValue, value: TypedValue) {
        assert!(memory_addr.type_of_variable.is_reference(), "memory address must be a reference");
        self.builder.insert_store(memory_addr.value_id, value.value_id);
    }

    /// Creates a new function with the given name and id with inline type InlineType::Inline
    /// Sets the same runtime as the builder
    pub fn new_function(&mut self, name: String, function_id: Id<Function>) {
        // maybe use different inline type
        match self.runtime {
            RuntimeType::Acir(inline_type) => {
                self.builder.new_function(name, function_id, inline_type);
            }
            RuntimeType::Brillig(inline_type) => {
                self.builder.new_brillig_function(name, function_id, inline_type);
            }
        }
    }

    /// Inserts an import function with the given function id
    pub fn insert_import(&mut self, function: Id<Function>) -> Id<Value> {
        self.builder.import_function(function)
    }

    /// Inserts a call to the given function with the given arguments and result type
    pub fn insert_call(
        &mut self,
        function: Id<Value>,
        arguments: &[TypedValue],
        result_type: Type,
    ) -> Id<Value> {
        *self
            .builder
            .insert_call(
                function,
                arguments.iter().map(|i| i.value_id).collect(),
                vec![result_type.clone().into()],
            )
            .first()
            .unwrap()
    }

    /// Inserts a new array with the given elements and type
    pub fn insert_array(&mut self, elements: Vec<TypedValue>) -> TypedValue {
        let array_length = elements.len() as u32;
        assert!(array_length > 0, "Array must have at least one element");
        let element_type = elements[0].type_of_variable.clone();
        assert!(
            elements.iter().all(|e| e.type_of_variable == element_type),
            "All elements must have the same type"
        );
        let array_elements_type = Type::Array(Arc::new(vec![element_type]), array_length);
        let res = self.builder.insert_make_array(
            elements.into_iter().map(|e| e.value_id).collect(),
            array_elements_type.clone().into(),
        );
        TypedValue::new(res, array_elements_type)
    }

    pub fn insert_vector(&mut self, elements: Vec<TypedValue>) -> TypedValue {
        let element_type = elements[0].type_of_variable.clone();
        assert!(
            elements.iter().all(|e| e.type_of_variable == element_type),
            "All elements must have the same type"
        );
        let array_elements_type = Type::Vector(Arc::new(vec![element_type]));
        let res = self.builder.insert_make_array(
            elements.into_iter().map(|e| e.value_id).collect(),
            array_elements_type.clone().into(),
        );
        TypedValue::new(res, array_elements_type)
    }

    /// Inserts a to_le_radix intrinsic call that decomposes a field into little-endian radix representation
    ///
    /// # Arguments
    /// * `field_value` - The field value to decompose (must be a field type)
    /// * `radix` - The radix to use (must be a power of 2, between 2 and 256)
    /// * `limb_count` - Number of limbs in the result array
    ///
    /// # Returns
    /// An array of u8 values representing the little-endian radix decomposition
    pub fn insert_to_le_radix(
        &mut self,
        field_value: TypedValue,
        radix: u32,
        limb_count: u8,
    ) -> TypedValue {
        assert!(field_value.is_field(), "to_le_radix requires a field value as input");
        let radix = self.builder.numeric_constant(radix, NumericType::U32.into());
        let intrinsic = self
            .builder
            .import_intrinsic("to_le_radix")
            .expect("to_le_radix intrinsic should be available");
        let element_type = Type::Numeric(NumericType::U8);
        let result_type = Type::Array(Arc::new(vec![element_type.clone()]), u32::from(limb_count));
        let result = self.builder.insert_call(
            intrinsic,
            vec![field_value.value_id, radix],
            vec![result_type.clone().into()],
        );

        TypedValue::new(result[0], result_type)
    }

    /// Inserts a from_le_radix hand-written function that composes a field from little-endian radix representation
    ///
    /// # Arguments
    /// * `array` - The array of u8 values to compose into a field
    /// * `radix` - The radix to use (must be a power of 2, between 2 and 256)
    ///
    /// # Returns
    /// A field value representing the composed value
    pub fn insert_from_le_radix(&mut self, array: TypedValue, radix: u128) -> TypedValue {
        let array_size = match array.type_of_variable {
            Type::Array(type_of_array, array_size) => {
                assert!(array_size > 0, "Array must have at least one element");
                assert!(matches!(type_of_array[0], Type::Numeric(NumericType::U8)));
                array_size
            }
            _ => unreachable!("Array type expected"),
        };
        let mut exp = self.builder.numeric_constant(1_u32, NumericType::Field.into());
        let mut agg = self.builder.numeric_constant(0_u32, NumericType::Field.into());
        let radix = self.builder.numeric_constant(radix, NumericType::Field.into());
        for i in 0..array_size {
            let index = self.builder.numeric_constant(i, NumericType::U32.into());
            let byte = self.builder.insert_array_get(
                array.value_id,
                index,
                SsaType::Numeric(NumericType::U8.into()),
            );
            let byte_as_field = self.builder.insert_cast(byte, NumericType::Field.into());
            let byte_as_field_mul_exp =
                self.builder.insert_binary(byte_as_field, BinaryOp::Mul { unchecked: true }, exp);
            agg = self.builder.insert_binary(
                agg,
                BinaryOp::Add { unchecked: true },
                byte_as_field_mul_exp,
            );
            exp = self.builder.insert_binary(exp, BinaryOp::Mul { unchecked: true }, radix);
        }
        TypedValue::new(agg, Type::Numeric(NumericType::Field))
    }

    /// Inserts a blake2s hash intrinsic call
    ///
    /// # Arguments
    /// * `input` - The array of u8 values to hash
    ///
    /// # Returns
    /// An array of u8 values representing the blake2s hash
    pub fn insert_blake2s_hash(&mut self, input: TypedValue) -> TypedValue {
        match input.type_of_variable {
            Type::Array(type_of_array, _array_size) => {
                assert!(
                    matches!(type_of_array[0], Type::Numeric(NumericType::U8)),
                    "blake2s requires an array of u8 as input"
                );
            }
            _ => unreachable!("blake2s requires an array as input"),
        }
        let intrinsic = self
            .builder
            .import_intrinsic("blake2s")
            .expect("blake2s intrinsic should be available");
        let return_type = Type::Array(Arc::new(vec![Type::Numeric(NumericType::U8)]), 32);
        let result = self.builder.insert_call(
            intrinsic,
            vec![input.value_id],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        TypedValue::new(result[0], return_type)
    }

    /// Inserts a blake3 hash intrinsic call
    ///
    /// # Arguments
    /// * `input` - The array of u8 values to hash
    ///
    /// # Returns
    /// An array of u8 values representing the blake3 hash
    pub fn insert_blake3_hash(&mut self, input: TypedValue) -> TypedValue {
        match input.type_of_variable {
            Type::Array(type_of_array, _array_size) => {
                assert!(matches!(type_of_array[0], Type::Numeric(NumericType::U8)));
            }
            _ => unreachable!("blake3 requires an array as input"),
        }
        let intrinsic =
            self.builder.import_intrinsic("blake3").expect("blake3 intrinsic should be available");
        let return_type = Type::Array(Arc::new(vec![Type::Numeric(NumericType::U8)]), 32);
        let result = self.builder.insert_call(
            intrinsic,
            vec![input.value_id],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        TypedValue::new(result[0], return_type)
    }

    /// Inserts a keccakf1600 permutation intrinsic call
    ///
    /// # Arguments
    /// * `input` - The array of u64 values to permute
    ///
    /// # Returns
    /// An array of u64 values representing the keccakf1600 permutation
    pub fn insert_keccakf1600_permutation(&mut self, input: TypedValue) -> TypedValue {
        match input.type_of_variable {
            Type::Array(type_of_array, array_size) => {
                assert!(
                    matches!(type_of_array[0], Type::Numeric(NumericType::U64)),
                    "keccakf1600 requires an array of u64 as input, but received {type_of_array:?}"
                );
                assert!(array_size == 25);
            }
            _ => unreachable!("keccakf1600 requires an array as input"),
        }
        let intrinsic = self
            .builder
            .import_intrinsic("keccakf1600")
            .expect("keccakf1600 intrinsic should be available");
        let return_type = Type::Array(Arc::new(vec![Type::Numeric(NumericType::U64)]), 25);
        let result = self.builder.insert_call(
            intrinsic,
            vec![input.value_id],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        TypedValue::new(result[0], return_type)
    }

    /// Inserts a aes128_encrypt intrinsic call
    ///
    /// # Arguments
    /// * `input` - The array of u8 values to encrypt
    /// * `key` - The array of u8 values to use as key must be 16 bytes
    /// * `iv` - The array of u8 values to use as iv must be 16 bytes
    ///
    /// # Returns
    /// An array of u8 values representing the encrypted input
    pub fn insert_aes128_encrypt(
        &mut self,
        input: TypedValue,
        key: TypedValue,
        iv: TypedValue,
    ) -> TypedValue {
        let input_size = match input.type_of_variable {
            Type::Array(type_of_array, array_size) => {
                assert!(matches!(type_of_array[0], Type::Numeric(NumericType::U8)));
                array_size
            }
            _ => unreachable!("aes128_encrypt requires an array as input"),
        };
        match key.type_of_variable {
            Type::Array(type_of_array, array_size) => {
                assert!(matches!(type_of_array[0], Type::Numeric(NumericType::U8)));
                assert!(array_size == 16);
            }
            _ => unreachable!("aes128_encrypt requires an array as key"),
        }
        match iv.type_of_variable {
            Type::Array(type_of_array, array_size) => {
                assert!(matches!(type_of_array[0], Type::Numeric(NumericType::U8)));
                assert!(array_size == 16);
            }
            _ => unreachable!("aes128_encrypt requires an array as iv"),
        }
        let intrinsic = self
            .builder
            .import_intrinsic("aes128_encrypt")
            .expect("aes128_encrypt intrinsic should be available");
        let return_type = Type::Array(
            Arc::new(vec![Type::Numeric(NumericType::U8)]),
            input_size + 16 - (input_size % 16),
        );
        let result = self.builder.insert_call(
            intrinsic,
            vec![input.value_id, key.value_id, iv.value_id],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        TypedValue::new(result[0], return_type)
    }

    /// Inserts a sha256 compression intrinsic call
    ///
    /// # Arguments
    /// * `input` - The array of u32 values to compress must be 16 elements
    /// * `state` - The array of u32 values to use as state must be 8 elements
    ///
    /// # Returns
    /// An array of u32 values representing the compressed input (8 elements)
    pub fn insert_sha256_compression(
        &mut self,
        input: TypedValue,
        state: TypedValue,
    ) -> TypedValue {
        match input.type_of_variable {
            Type::Array(type_of_array, array_size) => {
                assert!(matches!(type_of_array[0], Type::Numeric(NumericType::U32)));
                assert!(array_size == 16);
            }
            _ => unreachable!("sha256_compression requires an array as input"),
        }
        match state.type_of_variable {
            Type::Array(type_of_array, array_size) => {
                assert!(matches!(type_of_array[0], Type::Numeric(NumericType::U32)));
                assert!(array_size == 8);
            }
            _ => unreachable!("sha256_compression requires an array as state"),
        }
        let return_type = Type::Array(Arc::new(vec![Type::Numeric(NumericType::U32)]), 8);
        let intrinsic = self
            .builder
            .import_intrinsic("sha256_compression")
            .expect("sha256_compression intrinsic should be available");
        let result = self.builder.insert_call(
            intrinsic,
            vec![input.value_id, state.value_id],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        TypedValue::new(result[0], return_type)
    }

    /// Inserts a modulo operation between the index and the array length
    /// Returns the id of the index modulo the array length
    fn insert_index_mod_array_length(
        &mut self,
        index: TypedValue,
        array: TypedValue,
    ) -> TypedValue {
        match array.type_of_variable.clone() {
            Type::Array(_, array_length) => {
                let array_length_id =
                    self.builder.numeric_constant(array_length, NumericType::U32.into());
                let index_mod_length =
                    self.builder.insert_binary(index.value_id, BinaryOp::Mod, array_length_id);
                TypedValue::new(index_mod_length, Type::Numeric(NumericType::U32))
            }
            _ => unreachable!("Array type expected"),
        }
    }

    /// Inserts an array get instruction
    ///
    /// Index must be u32
    /// If safe_index is true, index will be taken modulo the array length
    pub fn insert_array_get(
        &mut self,
        array: TypedValue,
        index: TypedValue,
        element_type: Type,
        safe_index: bool,
    ) -> TypedValue {
        assert!(index.type_of_variable == Type::Numeric(NumericType::U32));
        let index = if safe_index {
            self.insert_index_mod_array_length(index.clone(), array.clone())
        } else {
            index
        };
        let res = self.builder.insert_array_get(
            array.value_id,
            index.value_id,
            element_type.clone().into(),
        );
        TypedValue::new(res, element_type)
    }

    /// Inserts an array set instruction
    ///
    /// Index must be u32
    /// If safe_index is true, index will be taken modulo the array length
    pub fn insert_array_set(
        &mut self,
        array: TypedValue,
        index: TypedValue,
        value: TypedValue,
        safe_index: bool,
    ) -> TypedValue {
        let (array_type, array_length) = match array.type_of_variable.clone() {
            Type::Array(array_type, array_length) => (array_type, array_length),
            _ => unreachable!("Array type expected"),
        };

        assert!(index.type_of_variable == Type::Numeric(NumericType::U32));
        let index = if safe_index {
            self.insert_index_mod_array_length(index.clone(), array.clone())
        } else {
            index
        };
        let res =
            self.builder.insert_array_set(array.value_id, index.value_id, value.value_id, false);
        TypedValue::new(res, Type::Array(array_type.clone(), array_length))
    }

    /// Performs a curve point multiplication with a scalar
    ///
    /// # Arguments
    /// * `scalar` - The scalar value to multiply
    /// * `is_infinite` - The boolean value indicating if the resulting point is on the curve (setting it by ourself)
    ///
    /// # Returns
    /// [`Point`] corresponding to `value` * G with overwritten on_curve value
    pub fn base_scalar_mul(&mut self, scalar: Scalar, is_infinite: TypedValue) -> Point {
        assert!(scalar.validate());
        assert!(matches!(is_infinite.type_of_variable, Type::Numeric(NumericType::Boolean)));
        let field_type = Type::Numeric(NumericType::Field);
        let boolean_type = Type::Numeric(NumericType::Boolean);
        let intrinsic = self
            .builder
            .import_intrinsic("multi_scalar_mul")
            .expect("multi_scalar_mul intrinsic should be available");

        // im recreating the point G every time, could be optimized to only do it once
        let g_x_id = self.builder.numeric_constant(1_u32, NumericType::Field.into());
        let g_y_id = self.builder.numeric_constant(
            FieldElement::try_from_str(
                "17631683881184975370165255887551781615748388533673675138860",
            )
            .unwrap(),
            NumericType::Field.into(),
        );
        let is_infinite_g_id = self.builder.numeric_constant(0_u32, NumericType::Boolean.into());
        let elements = vec![g_x_id, g_y_id, is_infinite_g_id].into_iter().collect();
        let basic_point = self.builder.insert_make_array(
            elements,
            SsaType::Array(
                Arc::new(vec![
                    field_type.clone().into(),
                    field_type.clone().into(),
                    boolean_type.clone().into(),
                ]),
                1,
            ),
        );
        let scalar_id = self.builder.insert_make_array(
            vec![scalar.lo.value_id, scalar.hi.value_id].into_iter().collect(),
            SsaType::Array(Arc::new(vec![field_type.clone().into(), field_type.clone().into()]), 1),
        );
        let return_type = Type::Array(
            Arc::new(vec![field_type.clone(), field_type.clone(), boolean_type.clone()]),
            1,
        );
        let predicate = self.builder.numeric_constant(1_u32, NumericType::Boolean.into());
        let result = self.builder.insert_call(
            intrinsic,
            vec![basic_point, scalar_id, predicate],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        let result = result[0];
        let x_idx = self.builder.numeric_constant(0_u32, NumericType::U32.into());
        let y_idx = self.builder.numeric_constant(1_u32, NumericType::U32.into());
        let x = self.builder.insert_array_get(result, x_idx, field_type.clone().into());
        let y = self.builder.insert_array_get(result, y_idx, field_type.clone().into());
        Point {
            x: TypedValue::new(x, field_type.clone()),
            y: TypedValue::new(y, field_type),
            is_infinite,
        }
    }

    /// Creates a point from an affine x coordinate (scalar.lo, scalar.hi, is_infinite)
    /// Mostly invalid
    /// # Arguments
    /// * `scalar` - The scalar value to take coordinates from
    /// * `is_infinite` - The boolean value indicating if the resulting point is on the curve (setting it by ourself)
    /// # Returns
    /// Point (scalar.lo, scalar.hi, is_infinite)
    pub fn create_point_from_scalar(&mut self, scalar: Scalar, is_infinite: TypedValue) -> Point {
        assert!(scalar.validate());
        assert!(matches!(is_infinite.type_of_variable, Type::Numeric(NumericType::Boolean)));
        Point { x: scalar.lo, y: scalar.hi, is_infinite }
    }

    pub fn multi_scalar_mul(
        &mut self,
        points: Vec<Point>,
        scalars: Vec<Scalar>,
        predicate: bool,
    ) -> Point {
        assert_eq!(points.len(), scalars.len());
        for point in &points {
            assert!(point.validate());
        }
        for scalar in &scalars {
            assert!(scalar.validate());
        }
        let predicate =
            self.builder.numeric_constant(u32::from(predicate), NumericType::Boolean.into());
        let field_type = Type::Numeric(NumericType::Field);
        let boolean_type = Type::Numeric(NumericType::Boolean);
        let intrinsic = self
            .builder
            .import_intrinsic("multi_scalar_mul")
            .expect("multi_scalar_mul intrinsic should be available");
        let point_ids = points.iter().flat_map(|p| p.to_id_vec()).collect::<Vec<_>>();
        let scalar_ids = scalars.iter().flat_map(|s| s.to_id_vec()).collect::<Vec<_>>();
        let point_ids_array = self.builder.insert_make_array(
            point_ids.into_iter().collect(),
            SsaType::Array(
                Arc::new(vec![
                    field_type.clone().into(),
                    field_type.clone().into(),
                    boolean_type.clone().into(),
                ]),
                points.len() as u32,
            ),
        );
        let scalar_ids_array = self.builder.insert_make_array(
            scalar_ids.into_iter().collect(),
            SsaType::Array(
                Arc::new(vec![field_type.clone().into(), field_type.clone().into()]),
                scalars.len() as u32,
            ),
        );
        let return_type = Type::Array(
            Arc::new(vec![field_type.clone(), field_type.clone(), boolean_type.clone()]),
            1,
        );
        let result = self.builder.insert_call(
            intrinsic,
            vec![point_ids_array, scalar_ids_array, predicate],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        let result = result[0];
        let x_idx = self.builder.numeric_constant(0_u32, NumericType::U32.into());
        let y_idx = self.builder.numeric_constant(1_u32, NumericType::U32.into());
        let is_infinite_idx = self.builder.numeric_constant(2_u32, NumericType::U32.into());
        let x = self.builder.insert_array_get(result, x_idx, field_type.clone().into());
        let y = self.builder.insert_array_get(result, y_idx, field_type.clone().into());
        let is_infinite =
            self.builder.insert_array_get(result, is_infinite_idx, boolean_type.clone().into());
        Point {
            x: TypedValue::new(x, field_type.clone()),
            y: TypedValue::new(y, field_type),
            is_infinite: TypedValue::new(is_infinite, boolean_type),
        }
    }

    pub fn point_add(&mut self, p1: Point, p2: Point, predicate: bool) -> Point {
        assert!(p1.validate());
        assert!(p2.validate());
        let field_type = Type::Numeric(NumericType::Field);
        let boolean_type = Type::Numeric(NumericType::Boolean);
        let predicate =
            self.builder.numeric_constant(u32::from(predicate), NumericType::Boolean.into());
        let intrinsic = self
            .builder
            .import_intrinsic("embedded_curve_add")
            .expect("embedded_curve_add intrinsic should be available");
        let mut arguments = p1.to_id_vec().into_iter().chain(p2.to_id_vec()).collect::<Vec<_>>();
        arguments.push(predicate);
        let return_type = Type::Array(
            Arc::new(vec![field_type.clone(), field_type.clone(), boolean_type.clone()]),
            1,
        );
        let result =
            self.builder.insert_call(intrinsic, arguments, vec![return_type.clone().into()]);
        assert_eq!(result.len(), 1);
        let result = result[0];
        let x_idx = self.builder.numeric_constant(0_u32, NumericType::U32.into());
        let y_idx = self.builder.numeric_constant(1_u32, NumericType::U32.into());
        let is_infinite_idx = self.builder.numeric_constant(2_u32, NumericType::U32.into());
        let x = self.builder.insert_array_get(result, x_idx, field_type.clone().into());
        let y = self.builder.insert_array_get(result, y_idx, field_type.clone().into());
        let is_infinite =
            self.builder.insert_array_get(result, is_infinite_idx, boolean_type.clone().into());
        Point {
            x: TypedValue::new(x, field_type.clone()),
            y: TypedValue::new(y, field_type),
            is_infinite: TypedValue::new(is_infinite, boolean_type),
        }
    }

    fn bytes_to_ssa_array(&mut self, vec: Vec<u8>) -> TypedValue {
        let elements: Vec<Id<Value>> = vec
            .into_iter()
            .map(|x| self.builder.numeric_constant(u32::from(x), NumericType::U8.into()))
            .collect();
        let array_type =
            Type::Array(Arc::new(vec![Type::Numeric(NumericType::U8)]), elements.len() as u32);
        TypedValue::new(
            self.builder.insert_make_array(elements.into(), array_type.clone().into()),
            array_type,
        )
    }

    pub fn ecdsa_secp256r1(
        &mut self,
        pub_key_x: Vec<u8>,
        pub_key_y: Vec<u8>,
        hash: Vec<u8>,
        signature: Vec<u8>,
        predicate: bool,
    ) -> TypedValue {
        let predicate =
            self.builder.numeric_constant(u32::from(predicate), NumericType::Boolean.into());
        let pub_key_x = self.bytes_to_ssa_array(pub_key_x);
        let pub_key_y = self.bytes_to_ssa_array(pub_key_y);
        let hash = self.bytes_to_ssa_array(hash);
        let signature = self.bytes_to_ssa_array(signature);
        let return_type = Type::Numeric(NumericType::Boolean);
        let intrinsic = self
            .builder
            .import_intrinsic("ecdsa_secp256r1")
            .expect("ecdsa_secp256r1 intrinsic should be available");
        let result = self.builder.insert_call(
            intrinsic,
            vec![
                pub_key_x.value_id,
                pub_key_y.value_id,
                signature.value_id,
                hash.value_id,
                predicate,
            ],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        let result = result[0];
        TypedValue::new(result, return_type)
    }

    pub fn ecdsa_secp256k1(
        &mut self,
        pub_key_x: Vec<u8>,
        pub_key_y: Vec<u8>,
        hash: Vec<u8>,
        signature: Vec<u8>,
        predicate: bool,
    ) -> TypedValue {
        let predicate =
            self.builder.numeric_constant(u32::from(predicate), NumericType::Boolean.into());
        let pub_key_x = self.bytes_to_ssa_array(pub_key_x);
        let pub_key_y = self.bytes_to_ssa_array(pub_key_y);
        let hash = self.bytes_to_ssa_array(hash);
        let signature = self.bytes_to_ssa_array(signature);
        let return_type = Type::Numeric(NumericType::Boolean);
        let intrinsic = self
            .builder
            .import_intrinsic("ecdsa_secp256k1")
            .expect("ecdsa_secp256k1 intrinsic should be available");
        let result = self.builder.insert_call(
            intrinsic,
            vec![
                pub_key_x.value_id,
                pub_key_y.value_id,
                signature.value_id,
                hash.value_id,
                predicate,
            ],
            vec![return_type.clone().into()],
        );
        assert_eq!(result.len(), 1);
        let result = result[0];
        TypedValue::new(result, return_type)
    }
}
