//! Validator that checks whether a function is well formed.
//!
//! It validates:
//!
//! SSA form
//!
//! - That the function contains exactly one return block.
//! - That every checked signed addition or subtraction instruction is
//!   followed by a corresponding truncate instruction with the expected bit sizes.
//!
//! Type checking
//! - Check that the input values of certain instructions matches that instruction's constraint
//!   At the moment, only [Instruction::Binary], [Instruction::ArrayGet], and [Instruction::ArraySet]
//!   are type checked.
use core::panic;
use std::sync::Arc;

use acvm::{AcirField, FieldElement, acir::BlackBoxFunc};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

pub(crate) mod dynamic_array_indices;

use crate::ssa::{
    ir::{basic_block::BasicBlockId, dfg::DataFlowGraph, instruction::TerminatorInstruction},
    ssa_gen::Ssa,
};

use super::ir::{
    function::Function,
    instruction::{Binary, BinaryOp, Instruction, InstructionId, Intrinsic},
    types::{NumericType, Type},
    value::{Value, ValueId},
};

/// Aside the function being validated, the validator maintains internal state
/// during instruction visitation to track patterns that span multiple instructions.
struct Validator<'f> {
    function: &'f Function,
    ssa: &'f Ssa,

    // State for valid Field to integer casts
    // Range checks are laid down in isolation and can make for safe casts
    // If they occurred before the value being cast to a smaller type
    // Stores: A set of (value being range constrained, the value's max bit size)
    range_checks: HashMap<ValueId, u32>,
}

impl<'f> Validator<'f> {
    fn new(function: &'f Function, ssa: &'f Ssa) -> Self {
        Self { function, ssa, range_checks: HashMap::default() }
    }

    /// Enforces that every cast from Field -> unsigned/signed integer must obey the following invariants:
    /// The value being cast is either:
    /// 1. A truncate instruction that ensures the cast is valid
    /// 2. A constant value known to be in-range
    /// 3. A division or other operation whose result is known to fit within the target bit size
    ///
    /// Our initial SSA gen only generates preceding truncates for safe casts.
    /// The cases accepted here are extended past what we perform during our initial SSA gen
    /// to mirror the instruction simplifier and other logic that could be accepted as a safe cast.
    fn validate_field_to_integer_cast_invariant(&mut self, instruction_id: InstructionId) {
        let dfg = &self.function.dfg;

        let (cast_input, typ) = match &dfg[instruction_id] {
            Instruction::Cast(cast_input, typ) => (*cast_input, *typ),
            Instruction::RangeCheck { value, max_bit_size, .. } => {
                self.range_checks.insert(*value, *max_bit_size);
                return;
            }
            _ => return,
        };

        if !matches!(dfg.type_of_value(cast_input), Type::Numeric(NumericType::NativeField)) {
            return;
        }

        let (NumericType::Signed { bit_size: target_type_size }
        | NumericType::Unsigned { bit_size: target_type_size }) = typ
        else {
            return;
        };

        // If the cast input has already been range constrained to a bit size that fits
        // in the destination type, we have a safe cast.
        if let Some(max_bit_size) = self.range_checks.get(&cast_input) {
            assert!(*max_bit_size <= target_type_size);
            return;
        }

        match &dfg[cast_input] {
            Value::Instruction { instruction, .. } => match &dfg[*instruction] {
                Instruction::Truncate { value: _, bit_size, max_bit_size } => {
                    assert!(*bit_size <= target_type_size);
                    assert!(*max_bit_size <= FieldElement::max_num_bits());
                }
                Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Div, .. })
                    if dfg.is_constant(*rhs) =>
                {
                    let numerator_bits = dfg.type_of_value(*lhs).bit_size();
                    let divisor = dfg.get_numeric_constant(*rhs).unwrap();
                    let divisor_bits = divisor.num_bits();
                    let max_quotient_bits = numerator_bits - divisor_bits;

                    assert!(
                        max_quotient_bits <= target_type_size,
                        "Cast from field after div could exceed bit size: expected ≤ {target_type_size}, got {max_quotient_bits}"
                    );
                }
                _ => {
                    panic!("Invalid cast from Field, must be truncated or provably safe");
                }
            },
            Value::NumericConstant { constant, .. } => {
                let max_val_bits = constant.num_bits();
                assert!(
                    max_val_bits <= target_type_size,
                    "Constant too large for cast target: {max_val_bits} bits > {target_type_size}"
                );
            }
            _ => {
                panic!(
                    "Invalid cast from Field, not preceded by valid truncation or known safe value"
                );
            }
        }
    }

    // Validates there is exactly one return block
    fn validate_single_return_block(&self) {
        let reachable_blocks = self.function.reachable_blocks();

        let return_blocks: HashSet<_> = reachable_blocks
            .iter()
            .filter(|block| {
                let terminator = self.function.dfg[**block].terminator().unwrap_or_else(|| {
                    panic!("Function {} has no terminator in block {block}", self.function.id())
                });
                matches!(terminator, TerminatorInstruction::Return { .. })
            })
            .collect();

        if return_blocks.len() > 1 {
            panic!("Function {} has multiple return blocks {return_blocks:?}", self.function.id())
        }
    }

    /// Validates that the instruction has the expected types associated with the values in each instruction
    fn type_check_instruction(&self, instruction: InstructionId) {
        let dfg = &self.function.dfg;
        match &dfg[instruction] {
            Instruction::Binary(Binary { lhs, rhs, operator }) => {
                let lhs_type = dfg.type_of_value(*lhs);
                let rhs_type = dfg.type_of_value(*rhs);

                assert_eq!(
                    lhs_type, rhs_type,
                    "Left-hand side and right-hand side of `{operator}` must have the same type"
                );

                if lhs_type == Type::field()
                    && matches!(
                        operator,
                        BinaryOp::Lt
                            | BinaryOp::And
                            | BinaryOp::Or
                            | BinaryOp::Xor
                            | BinaryOp::Shl
                            | BinaryOp::Shr
                    )
                {
                    panic!("Cannot use `{operator}` with field elements");
                };
            }
            Instruction::ArrayGet { array, index, .. }
            | Instruction::ArraySet { array, index, .. } => {
                let index_type = dfg.type_of_value(*index);
                if !matches!(index_type, Type::Numeric(NumericType::Unsigned { bit_size: 32 })) {
                    panic!("ArrayGet/ArraySet index must be u32");
                }
                let array_type = dfg.type_of_value(*array);
                if !array_type.contains_an_array() {
                    panic!("ArrayGet/ArraySet must operate on an array; got {array_type}");
                }
                assert!(!array_type.is_nested_vector(), "ICE: Nested vector type is not supported");
                let instruction_results = dfg.instruction_results(instruction);
                for result in instruction_results {
                    let return_type = dfg.type_of_value(*result);
                    assert!(
                        !return_type.is_nested_vector(),
                        "ICE: Nested vector type is not supported"
                    );
                }
            }
            Instruction::Call { func, arguments } => {
                self.type_check_call(instruction, func, arguments);
            }
            Instruction::Constrain(lhs, rhs, _) | Instruction::ConstrainNotEqual(lhs, rhs, _) => {
                let lhs_type = dfg.type_of_value(*lhs);
                let rhs_type = dfg.type_of_value(*rhs);
                if lhs_type != rhs_type {
                    panic!(
                        "Left-hand side and right-hand side of constrain must have the same type"
                    );
                }
            }
            Instruction::MakeArray { elements, typ: _ } => {
                let result_type = self.assert_one_result(instruction, "MakeArray");

                let composite_type = match result_type {
                    Type::Array(composite_type, length) => {
                        let array_flattened_length = composite_type.len() * length as usize;
                        if elements.len() != array_flattened_length {
                            panic!(
                                "MakeArray returns an array of flattened length {}, but it has {} elements",
                                array_flattened_length,
                                elements.len()
                            );
                        }
                        composite_type
                    }
                    Type::Vector(composite_type) => {
                        if composite_type.is_empty() {
                            if !elements.is_empty() {
                                panic!(
                                    "MakeArray vector has non-zero {} elements but composite type is empty",
                                    elements.len(),
                                );
                            }
                        } else if elements.len() % composite_type.len() != 0 {
                            panic!(
                                "MakeArray vector has {} elements but composite type has {} types which don't divide the number of elements",
                                elements.len(),
                                composite_type.len()
                            );
                        }
                        composite_type
                    }
                    _ => {
                        panic!("MakeArray must return an array or vector type, not {result_type}");
                    }
                };

                let composite_type_len = composite_type.len();
                for (index, element) in elements.iter().enumerate() {
                    let element_type = dfg.type_of_value(*element);
                    let expected_type = &composite_type[index % composite_type_len];
                    if &element_type != expected_type {
                        panic!(
                            "MakeArray has incorrect element type at index {index}: expected {}, got {}",
                            expected_type, element_type
                        );
                    }
                }
            }
            Instruction::Store { address, value } => {
                let address_type = dfg.type_of_value(*address);
                let Type::Reference(address_value_type) = address_type else {
                    panic!("Store address must be a reference type, got {address_type}");
                };

                let value_type = dfg.type_of_value(*value);
                if *address_value_type != value_type {
                    panic!(
                        "Store address type {} does not match value type {}",
                        address_value_type, value_type
                    );
                }
            }
            _ => (),
        }
    }

    fn type_check_call(&self, instruction: InstructionId, func: &ValueId, arguments: &[ValueId]) {
        let dfg = &self.function.dfg;
        match &dfg[*func] {
            Value::Intrinsic(intrinsic) => {
                self.type_check_intrinsic(instruction, arguments, intrinsic);
            }
            Value::Function(func_id) => {
                let called_function = &self.ssa.functions[func_id];

                let parameter_types = called_function.view().parameter_types();
                assert_eq!(
                    arguments.len(),
                    parameter_types.len(),
                    "Function call to {func_id} expected {} parameters, but got {}",
                    parameter_types.len(),
                    arguments.len()
                );

                for (index, (argument, parameter_type)) in
                    arguments.iter().zip(parameter_types).enumerate()
                {
                    let argument_type = dfg.type_of_value(*argument);
                    if argument_type != parameter_type {
                        panic!(
                            "Argument #{} to {func_id} has type {parameter_type}, but {argument_type} was given",
                            index + 1,
                        );
                    }
                }

                if let Some(returns) = called_function.returns() {
                    let instruction_results = dfg.instruction_results(instruction);
                    if instruction_results.len() != returns.len() {
                        panic!(
                            "Function call to {} expected {} return values, but got {}",
                            func_id,
                            instruction_results.len(),
                            returns.len(),
                        );
                    }
                    for (index, (instruction_result, return_value)) in
                        instruction_results.iter().zip(returns).enumerate()
                    {
                        let return_type = called_function.dfg.type_of_value(*return_value);
                        let instruction_result_type = dfg.type_of_value(*instruction_result);
                        if return_type != instruction_result_type {
                            panic!(
                                "Function call to {} expected return type {}, but got {} (at position {})",
                                func_id,
                                instruction_result_type,
                                return_type,
                                index + 1
                            );
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn type_check_intrinsic(
        &self,
        instruction: InstructionId,
        arguments: &[ValueId],
        intrinsic: &Intrinsic,
    ) {
        match intrinsic {
            Intrinsic::ToRadix(_) => {
                // fn __to_le_radix<let N: u32>(value: Field, radix: u32) -> [u8; N] {}
                // fn __to_be_radix<let N: u32>(value: Field, radix: u32) -> [u8; N] {}
                let (value_type, radix_type) = self.assert_two_arguments(arguments, "ToRadix");
                assert_field(&value_type, "ToRadix value");
                assert_u32(&radix_type, "ToRadix radix");

                let result_type = self.assert_one_result(instruction, "ToRadix");
                assert_u8_array(&result_type, "to_radix output");
            }
            Intrinsic::ToBits(_) => {
                // fn __to_le_bits<let N: u32>(value: Field) -> [u1; N] {}
                // fn __to_be_bits<let N: u32>(value: Field) -> [u1; N] {}
                let value_type = self.assert_one_argument(arguments, "ToBits");
                assert_field(&value_type, "ToBits value");

                let result_type = self.assert_one_result(instruction, "ToBits");
                assert_u1_array(&result_type, "to_bits output");
            }
            Intrinsic::ArrayLen => {
                // fn len(self: [T; N]) -> u32 {}
                let argument_type = self.assert_one_argument(arguments, "ArrayLen");
                assert_array(&argument_type, "ArrayLen argument");

                let result_type = self.assert_one_result(instruction, "ArrayLen");
                assert_u32(&result_type, "ArrayLen return");
            }
            Intrinsic::ArrayAsStrUnchecked => {
                // fn as_str_unchecked(self: [u8; N]) -> str<N> {}
                let argument_type = self.assert_one_argument(arguments, "ArrayAsStrUnchecked");
                let array_length = assert_u8_array(&argument_type, "ArrayAsStrUnchecked argument");

                let result_type = self.assert_one_result(instruction, "ArrayAsStrUnchecked");
                let string_length = assert_u8_array(&result_type, "ArrayAsStrUnchecked argument");
                assert_eq!(
                    array_length, string_length,
                    "ArrayAsStrUnchecked array length must match string length"
                );
            }
            Intrinsic::AsVector => {
                // fn as_vector(self: [T; N]) -> [T] {}
                let argument_type = self.assert_one_argument(arguments, "AsVector");
                let (array_types, _array_length) = assert_array(&argument_type, "AsVector argument");

                let results = self.function.dfg.instruction_results(instruction);
                assert_eq!(results.len(), 2, "Expected two results for AsVector",);

                let length_type = self.function.dfg.type_of_value(results[0]);
                assert_u32(&length_type, "AsVector length");

                let vector_type = self.function.dfg.type_of_value(results[1]);
                let vector_types = assert_vector(&vector_type, "AsVector return");
                assert_eq!(
                    array_types, vector_types,
                    "AsVector input array element types must match output vector element types"
                );
            }
            Intrinsic::AssertConstant => {
                // fn assert_constant<T>(x: T) {}
                self.assert_no_results(instruction, "AssertConstant");
            }
            Intrinsic::StaticAssert => {
                // fn static_assert<T>(predicate: bool, message: T) {}
                assert!(!arguments.is_empty(), "StaticAssert must have at least one argument");

                let predicate_type = self.function.dfg.type_of_value(arguments[0]);
                assert_u1(&predicate_type, "StaticAssert predicate");

                self.assert_no_results(instruction, "StaticAssert");
            }
            Intrinsic::VectorPushBack | Intrinsic::VectorPushFront => {
                // fn push_back(self: [T], elem: T) -> Self {}
                // fn push_front(self: [T], elem: T) -> Self {}
                assert!(arguments.len() >= 2, "VectorPush must have at least two arguments");

                let vector_length_type = self.function.dfg.type_of_value(arguments[0]);
                assert_u32(&vector_length_type, "VectorPush self length");

                let vector_type = self.function.dfg.type_of_value(arguments[1]);
                let vector_element_types = assert_vector(&vector_type, "VectorPush self vector");

                let (returned_vector_length_type, returned_vector_type) =
                    self.assert_two_results(instruction, "VectorPush");
                assert_u32(&returned_vector_length_type, "VectorPush returned length");
                let returned_vector_element_types =
                    assert_vector(&returned_vector_type, "VectorPush returned vector");
                assert_eq!(
                    vector_element_types, returned_vector_element_types,
                    "VectorPush self vector element types must match returned vector element types"
                );
            }
            Intrinsic::VectorPopBack => {
                // fn pop_back(self: [T]) -> (Self, T) {}
                let (vector_length_type, vector_type) =
                    self.assert_two_arguments(arguments, "VectorPopBack");
                assert_u32(&vector_length_type, "VectorPopBack self length");
                let vector_element_types = assert_vector(&vector_type, "VectorPopBack self vector");

                let results = self.function.dfg.instruction_results(instruction);
                assert!(results.len() >= 2, "Expected at least two results for VectorPopBack");

                let returned_vector_length_type = self.function.dfg.type_of_value(results[0]);
                assert_u32(&returned_vector_length_type, "VectorPopBack returned length");

                let returned_vector_type = self.function.dfg.type_of_value(results[1]);
                let returned_vector_element_types =
                    assert_vector(&returned_vector_type, "VectorPopBack returned vector");
                assert_eq!(
                    vector_element_types, returned_vector_element_types,
                    "VectorPopBack self vector element types must match returned vector element types"
                );
            }
            Intrinsic::VectorPopFront => {
                // fn pop_front(self: [T]) -> (T, Self) {}
                let (vector_length_type, vector_type) =
                    self.assert_two_arguments(arguments, "VectorPopFront");
                assert_u32(&vector_length_type, "VectorPopFront self length");
                let vector_element_types = assert_vector(&vector_type, "VectorPopFront self vector");

                let results = self.function.dfg.instruction_results(instruction);
                assert!(results.len() >= 2, "Expected at least two results for VectorPopFront");

                let returned_vector_type =
                    self.function.dfg.type_of_value(results[results.len() - 1]);
                let returned_vector_element_types =
                    assert_vector(&returned_vector_type, "VectorPopFront returned vector");
                assert_eq!(
                    vector_element_types, returned_vector_element_types,
                    "VectorPopFront self vector element types must match returned vector element types"
                );

                let returned_vector_length_type =
                    self.function.dfg.type_of_value(results[results.len() - 2]);
                assert_u32(&returned_vector_length_type, "VectorPopFront returned length");
            }
            Intrinsic::VectorInsert => {
                // fn insert(self: [T], index: u32, elem: T) -> Self {}
                assert!(arguments.len() >= 3, "VectorInsert must have at least three arguments");

                let vector_length_type = self.function.dfg.type_of_value(arguments[0]);
                assert_u32(&vector_length_type, "VectorInsert self length");

                let vector_type = self.function.dfg.type_of_value(arguments[1]);
                let vector_element_types = assert_vector(&vector_type, "VectorInsert self vector");

                let index_type = self.function.dfg.type_of_value(arguments[2]);
                assert_u32(&index_type, "VectorInsert index");

                let (returned_vector_length_type, returned_vector_type) =
                    self.assert_two_results(instruction, "VectorInsert");
                assert_u32(&returned_vector_length_type, "VectorInsert returned length");
                let returned_vector_element_types =
                    assert_vector(&returned_vector_type, "VectorInsert returned vector");
                assert_eq!(
                    vector_element_types, returned_vector_element_types,
                    "VectorInsert self vector element types must match returned vector element types"
                );
            }
            Intrinsic::VectorRemove => {
                // fn remove(self: [T], index: u32) -> (Self, T) {}
                let (vector_length_type, vector_type, index_type) =
                    self.assert_three_arguments(arguments, "VectorRemove");

                assert_u32(&vector_length_type, "VectorRemove self length");

                let vector_element_types = assert_vector(&vector_type, "VectorRemove self vector");

                assert_u32(&index_type, "VectorRemove index");

                let results = self.function.dfg.instruction_results(instruction);
                assert!(results.len() >= 2, "Expected at least two results for VectorRemove");

                let returned_vector_length_type = self.function.dfg.type_of_value(results[0]);
                assert_u32(&returned_vector_length_type, "VectorRemove returned length");

                let returned_vector_type = self.function.dfg.type_of_value(results[1]);
                let returned_vector_element_types =
                    assert_vector(&returned_vector_type, "VectorRemove returned vector");
                assert_eq!(
                    vector_element_types, returned_vector_element_types,
                    "VectorRemove self vector element types must match returned vector element types"
                );
            }
            Intrinsic::ApplyRangeConstraint => {
                // fn __assert_max_bit_size(value: Field, bit_size: u32) {}
                let (value_type, bit_size_type) =
                    self.assert_two_arguments(arguments, "ApplyRangeConstraint");
                assert_field(&value_type, "ApplyRangeConstraint value");
                assert_u32(&bit_size_type, "ApplyRangeConstraint bit size");
            }
            Intrinsic::StrAsBytes => {
                // fn as_bytes(self: str<N>) -> [u8; N] {}
                let argument_type = self.assert_one_argument(arguments, "StrAsBytes");
                let string_length = assert_u8_array(&argument_type, "StrAsBytes argument");

                let result_type = self.assert_one_result(instruction, "StrAsBytes");
                let array_length = assert_u8_array(&result_type, "StrAsBytes argument");
                assert_eq!(
                    string_length, array_length,
                    "StrAsBytes string length must match array length"
                );
            }
            Intrinsic::Hint(_) => {
                // fn black_box<T>(value: T) -> T {}
                // Not much we can do here (T might be `()` so arguments aren't guaranteed)
            }
            Intrinsic::AsWitness => {
                // fn as_witness(x: Field) {}
                let argument_type = self.assert_one_argument(arguments, "AsWitness");
                assert_field(&argument_type, "AsWitness argument");

                self.assert_no_results(instruction, "AsWitness");
            }
            Intrinsic::IsUnconstrained => {
                // fn is_unconstrained() -> bool {}
                assert_arguments_length(arguments, 0, "IsUnconstrained");

                let result_type = self.assert_one_result(instruction, "IsUnconstrained");
                assert_u1(&result_type, "IsUnconstrained result");
            }
            Intrinsic::DerivePedersenGenerators => {
                // fn __derive_generators<let N: u32, let M: u32>(
                //     domain_separator_bytes: [u8; M],
                //     starting_index: u32,
                // ) -> [EmbeddedCurvePoint; N] {}
                let (domain_separator_bytes_type, starting_index_type) =
                    self.assert_two_arguments(arguments, "DerivePedersenGenerators");
                assert_u8_array(
                    &domain_separator_bytes_type,
                    "DerivePedersenGenerators domain_separator_bytes",
                );
                assert_u32(&starting_index_type, "DerivePedersenGenerators starting_index");

                let result_type = self.assert_one_result(instruction, "DerivePedersenGenerators");
                let (result_elements, _array_length) =
                    assert_array(&result_type, "DerivePedersenGenerators result");
                assert_eq!(
                    result_elements.len(),
                    3,
                    "Expected embedded_curve_add result element types length to be 3, got: {}",
                    result_elements.len(),
                );
                assert_field(&result_elements[0], "embedded_curve_add result x");
                assert_field(&result_elements[1], "embedded_curve_add result y");
                assert_u1(&result_elements[2], "embedded_curve_add result is_infinite");
            }
            Intrinsic::FieldLessThan => {
                // fn __field_less_than(x: Field, y: Field) -> bool {}
                let (x_type, y_type) = self.assert_two_arguments(arguments, "FieldLessThan");
                assert_field(&x_type, "FieldLessThan x");
                assert_field(&y_type, "FieldLessThan y");

                let result_type = self.assert_one_result(instruction, "FieldLessThan");
                assert_u1(&result_type, "FieldLessThan result");
            }
            Intrinsic::ArrayRefCount => {
                // fn array_refcount<T, let N: u32>(array: [T; N]) -> u32 {}
                let array_type = self.assert_one_argument(arguments, "ArrayRefCount");
                assert_array(&array_type, "ArrayRefCount array");

                let result_type = self.assert_one_result(instruction, "ArrayRefCount");
                assert_u32(&result_type, "ArrayRefCount result");
            }
            Intrinsic::VectorRefCount => {
                // fn vector_refcount<T>(vector: [T]) -> u32 {}
                let (vector_length_type, vector_type) =
                    self.assert_two_arguments(arguments, "VectorRefCount");
                assert_u32(&vector_length_type, "VectorRefCount length");
                assert_vector(&vector_type, "VectorRefCount vector");

                let result_type = self.assert_one_result(instruction, "VectorRefCount");
                assert_u32(&result_type, "VectorRefCount result");
            }
            Intrinsic::BlackBox(blackbox) => {
                self.type_check_black_box(instruction, arguments, blackbox);
            }
        }
    }

    fn type_check_black_box(
        &self,
        instruction: InstructionId,
        arguments: &[ValueId],
        blackbox: &BlackBoxFunc,
    ) {
        let dfg = &self.function.dfg;
        match blackbox {
            BlackBoxFunc::AND | BlackBoxFunc::XOR => {
                assert_eq!(arguments.len(), 2);
                let value_typ = dfg.type_of_value(arguments[0]);
                assert!(
                    matches!(
                        value_typ,
                        Type::Numeric(NumericType::Unsigned { .. } | NumericType::Signed { .. })
                    ),
                    "Bitwise operation performed on non-integer type"
                );
            }
            BlackBoxFunc::AES128Encrypt => {
                // fn aes128_encrypt<let N: u32>(
                //     input: [u8; N],
                //     iv: [u8; 16],
                //     key: [u8; 16],
                // ) -> [u8; N + 16 - N % 16] {}
                let (input_type, iv_type, key_type) =
                    self.assert_three_arguments(arguments, "aes128_encrypt");

                let input_length = assert_u8_array(&input_type, "aes128_encrypt input");

                let iv_length = assert_u8_array(&iv_type, "aes128_encrypt iv");
                assert_array_length(iv_length, 16, "aes128_encrypt iv");

                let key_length = assert_u8_array(&key_type, "aes128_encrypt key");
                assert_array_length(key_length, 16, "aes128_encrypt key");

                let result_type = self.assert_one_result(instruction, "aes128_encrypt");
                let result_length = assert_u8_array(&result_type, "aes128_encrypt output");
                assert_eq!(
                    result_length,
                    input_length + 16 - input_length % 16,
                    "aes128_encrypt output length mismatch"
                );
            }
            BlackBoxFunc::Blake2s | BlackBoxFunc::Blake3 => {
                // fn blake2s<let N: u32>(input: [u8; N]) -> [u8; 32]
                // fn __blake3<let N: u32>(input: [u8; N]) -> [u8; 32] {}
                let input_type = self.assert_one_argument(arguments, "blake");
                assert_u8_array(&input_type, "blake input");

                let result_type = self.assert_one_result(instruction, "blake");
                let result_length = assert_u8_array(&result_type, "blake output");
                assert_array_length(result_length, 32, "blake output");
            }
            BlackBoxFunc::EcdsaSecp256k1 | BlackBoxFunc::EcdsaSecp256r1 => {
                // fn verify_signature<let N: u32>(
                //     public_key_x: [u8; 32],
                //     public_key_y: [u8; 32],
                //     signature: [u8; 64],
                //     message_hash: [u8; N],
                //     predicate: bool,
                // ) -> bool
                assert_arguments_length(arguments, 5, "ecdsa_secp_256");

                let public_key_x = arguments[0];
                let public_key_x_type = dfg.type_of_value(public_key_x);
                let public_key_x_length =
                    assert_u8_array(&public_key_x_type, "ecdsa_secp256 public_key_x");
                assert_array_length(public_key_x_length, 32, "ecdsa_secp256 public_key_x");

                let public_key_y = arguments[1];
                let public_key_y_type = dfg.type_of_value(public_key_y);
                let public_key_y_length =
                    assert_u8_array(&public_key_y_type, "ecdsa_secp256 public_key_y");
                assert_array_length(public_key_y_length, 32, "ecdsa_secp256 public_key_y");

                let signature = arguments[2];
                let signature_type = dfg.type_of_value(signature);
                let signature_length = assert_u8_array(&signature_type, "ecdsa_secp256 signature");
                assert_array_length(signature_length, 64, "ecdsa_secp256 signature");

                let message_hash = arguments[3];
                let message_hash_type = dfg.type_of_value(message_hash);
                assert_array(&message_hash_type, "ecdsa_secp256 message_hash");

                let predicate_type = dfg.type_of_value(arguments[4]);
                assert_u1(&predicate_type, "ecdsa_secp256 predicate");

                let results = dfg.instruction_results(instruction);
                assert_eq!(results.len(), 1, "Expected one result");
                let result_type = dfg.type_of_value(results[0]);
                assert_u1(&result_type, "ecdsa_secp256 result");
            }
            BlackBoxFunc::EmbeddedCurveAdd => {
                // fn embedded_curve_add_array_return(
                //     _point1: EmbeddedCurvePoint,
                //     _point2: EmbeddedCurvePoint,
                //     _predicate: bool,
                // ) -> [EmbeddedCurvePoint; 1] {}
                //
                // struct EmbeddedCurvePoint {
                //     x: Field,
                //     y: Field,
                //     is_infinite: bool,
                // }
                assert_arguments_length(arguments, 7, "embedded_curve_add");

                assert_embedded_curve_point(arguments, 0, dfg, "embedded_curve_add _point1");
                assert_embedded_curve_point(arguments, 3, dfg, "embedded_curve_add _point2");

                let predicate_type = dfg.type_of_value(arguments[6]);
                assert_u1(&predicate_type, "embedded_curve_add _predicate");

                let result_type = self.assert_one_result(instruction, "embedded_curve_add");
                let (result_elements, result_length) =
                    assert_array(&result_type, "embedded_curve_add result");
                assert_array_length(result_length, 1, "embedded_curve_add result length");
                assert_eq!(
                    result_elements.len(),
                    3,
                    "Expected embedded_curve_add result element types length to be 3, got: {}",
                    result_elements.len(),
                );
                assert_field(&result_elements[0], "embedded_curve_add result x");
                assert_field(&result_elements[1], "embedded_curve_add result y");
                assert_u1(&result_elements[2], "embedded_curve_add result is_infinite");
            }
            BlackBoxFunc::Keccakf1600 => {
                // fn keccakf1600(input: [u64; 25]) -> [u64; 25] {}
                let input_type = self.assert_one_argument(arguments, "keccakf1600");
                let input_length = assert_u64_array(&input_type, "keccakf1600 input");
                assert_array_length(input_length, 25, "keccakf1600 input");

                let results = dfg.instruction_results(instruction);
                assert_eq!(results.len(), 1);
                let result_type = dfg.type_of_value(results[0]);
                let result_length = assert_u64_array(&result_type, "keccakf1600 result");
                assert_array_length(result_length, 25, "keccakf1600 result");
            }
            BlackBoxFunc::MultiScalarMul => {
                //  fn multi_scalar_mul_array_return<let N: u32>(
                //     points: [EmbeddedCurvePoint; N],
                //     scalars: [EmbeddedCurveScalar; N],
                //     predicate: bool,
                // ) -> [EmbeddedCurvePoint; 1] {}
                let (points_type, scalars_type, predicate_type) =
                    self.assert_three_arguments(arguments, "multi_scalar_mul");

                let (points_elements, points_length) =
                    assert_array(&points_type, "multi_scalar_mul points");
                assert_eq!(
                    points_elements.len(),
                    3,
                    "Expected multi_scalar_mul points element types length to be 3, got: {}",
                    points_elements.len()
                );
                assert_field(&points_elements[0], "multi_scalar_mul points x");
                assert_field(&points_elements[1], "multi_scalar_mul points y");
                assert_u1(&points_elements[2], "multi_scalar_mul points is_infinite");

                let (scalars_elements, scalars_length) =
                    assert_array(&scalars_type, "multi_scalar_mul scalars");
                assert_eq!(
                    scalars_elements.len(),
                    2,
                    "Expected multi_scalar_mul scalars element types length to be 2, got: {}",
                    scalars_elements.len()
                );
                assert_field(&scalars_elements[0], "multi_scalar_mul scalars lo");
                assert_field(&scalars_elements[1], "multi_scalar_mul scalars hi");

                assert_eq!(points_length, scalars_length, "MSM input array lengths mismatch");

                assert_u1(&predicate_type, "multi_scalar_mul predicate");
            }
            BlackBoxFunc::Poseidon2Permutation => {
                // fn poseidon2_permutation<let N: u32>(_input: [Field; N]) -> [Field; N] {}
                let input_type = self.assert_one_argument(arguments, "poseidon2_permutation");
                let input_length = assert_field_array(&input_type, "poseidon2_permutation _input");

                let result_type = self.assert_one_result(instruction, "poseidon2_permutation");
                let result_length =
                    assert_field_array(&result_type, "poseidon2_permutation result");
                assert_eq!(
                    result_length, input_length,
                    "poseidon2_permutation input/output length mismatch"
                );
            }
            BlackBoxFunc::RANGE => {}
            BlackBoxFunc::RecursiveAggregation => {
                // fn verify_proof_internal<let N: u32, let M: u32, let K: u32>(
                //     verification_key: [Field; N],
                //     proof: [Field; M],
                //     public_inputs: [Field; K],
                //     key_hash: Field,
                //     proof_type: u32,
                // ) {}
                assert_arguments_length(arguments, 5, "recursive_aggregation");

                let verification_key = arguments[0];
                let verification_key_type = dfg.type_of_value(verification_key);
                assert_field_array(
                    &verification_key_type,
                    "recursive_aggregation verification_key",
                );

                let proof = arguments[1];
                let proof_type = dfg.type_of_value(proof);
                assert_field_array(&proof_type, "recursive_aggregation proof");

                let public_inputs = arguments[2];
                let public_inputs_type = dfg.type_of_value(public_inputs);
                assert_field_array(&public_inputs_type, "recursive_aggregation public_inputs");

                let key_hash = arguments[3];
                let key_hash_type = dfg.type_of_value(key_hash);
                assert_field(&key_hash_type, "recursive_aggregation key_hash");

                let proof_type = arguments[4];
                let proof_type_type = dfg.type_of_value(proof_type);
                assert_u32(&proof_type_type, "recursive_aggregation proof_type");

                self.assert_no_results(instruction, "recursive_aggregation");
            }
            BlackBoxFunc::Sha256Compression => {
                // fn sha256_compression(input: [u32; 16], state: [u32; 8]) -> [u32; 8] {}
                let (input_type, state_type) =
                    self.assert_two_arguments(arguments, "sha256_compression");

                let input_length = assert_u32_array(&input_type, "sha256_compression input");
                assert_array_length(input_length, 16, "sha256_compression input");

                let state_length = assert_u32_array(&state_type, "sha256_compression state");
                assert_array_length(state_length, 8, "sha256_compression state");

                let result_type = self.assert_one_result(instruction, "sha256_compression");
                let result_length = assert_u32_array(&result_type, "sha256_compression result");
                assert_array_length(result_length, 8, "sha256_compression result");
            }
        }
    }

    fn assert_one_argument(&self, arguments: &[ValueId], object: &'static str) -> Type {
        assert_arguments_length(arguments, 1, object);

        self.function.dfg.type_of_value(arguments[0])
    }

    fn assert_two_arguments(&self, arguments: &[ValueId], object: &'static str) -> (Type, Type) {
        assert_arguments_length(arguments, 2, object);

        (
            self.function.dfg.type_of_value(arguments[0]),
            self.function.dfg.type_of_value(arguments[1]),
        )
    }

    fn assert_three_arguments(
        &self,
        arguments: &[ValueId],
        object: &'static str,
    ) -> (Type, Type, Type) {
        assert_arguments_length(arguments, 3, object);

        (
            self.function.dfg.type_of_value(arguments[0]),
            self.function.dfg.type_of_value(arguments[1]),
            self.function.dfg.type_of_value(arguments[2]),
        )
    }

    fn assert_no_results(&self, instruction: InstructionId, object: &'static str) {
        let results = self.function.dfg.instruction_results(instruction);
        assert_eq!(results.len(), 0, "Expected zero result for {object}",);
    }

    fn assert_one_result(&self, instruction: InstructionId, object: &'static str) -> Type {
        let results = self.function.dfg.instruction_results(instruction);
        assert_eq!(results.len(), 1, "Expected one result for {object}",);
        self.function.dfg.type_of_value(results[0])
    }

    fn assert_two_results(&self, instruction: InstructionId, object: &'static str) -> (Type, Type) {
        let results = self.function.dfg.instruction_results(instruction);
        assert_eq!(results.len(), 2, "Expected two results for {object}",);
        (self.function.dfg.type_of_value(results[0]), self.function.dfg.type_of_value(results[1]))
    }

    /// Validates that ACIR functions are not called from unconstrained code.
    fn check_calls_in_unconstrained(&self, instruction: InstructionId) {
        if self.function.runtime().is_brillig() {
            if let Instruction::Call { func, .. } = &self.function.dfg[instruction] {
                if let Value::Function(func_id) = &self.function.dfg[*func] {
                    let called_function = &self.ssa.functions[func_id];
                    if called_function.runtime().is_acir() {
                        panic!(
                            "Call to ACIR function '{} {}' from unconstrained '{} {}'",
                            called_function.name(),
                            called_function.id(),
                            self.function.name(),
                            self.function.id(),
                        );
                    }
                }
            }
        }
    }

    /// Check the inputs and outputs of function calls going from ACIR to Brillig:
    /// * cannot pass references from constrained to unconstrained code
    /// * cannot return functions
    /// * cannot call oracles directly
    fn check_calls_in_constrained(&self, instruction: InstructionId) {
        if !self.function.runtime().is_acir() {
            return;
        }
        let Instruction::Call { func, arguments } = &self.function.dfg[instruction] else {
            return;
        };
        let callee_id = match &self.function.dfg[*func] {
            Value::Function(func_id) => func_id,
            Value::ForeignFunction(oracle) => {
                panic!(
                    "Trying to call foreign function '{oracle}' from ACIR function '{} {}'",
                    self.function.name(),
                    self.function.id()
                );
            }
            _ => return,
        };
        let called_function = &self.ssa.functions[callee_id];
        if called_function.runtime().is_acir() {
            return;
        }
        for arg_id in arguments {
            let typ = self.function.dfg.type_of_value(*arg_id);
            if typ.contains_reference() {
                // If we don't panic here, we would have a different, more obscure panic later on.
                panic!(
                    "Trying to pass a reference from ACIR function '{} {}' to unconstrained '{} {}' in argument {arg_id}: {typ}",
                    self.function.name(),
                    self.function.id(),
                    called_function.name(),
                    called_function.id()
                )
            }
        }
        for result_id in self.function.dfg.instruction_results(instruction) {
            let typ = self.function.dfg.type_of_value(*result_id);
            if typ.contains_function() {
                panic!(
                    "Trying to return a function value to ACIR function '{} {}' from unconstrained '{} {}' in {result_id}: {typ}",
                    self.function.name(),
                    self.function.id(),
                    called_function.name(),
                    called_function.id()
                )
            }
        }
    }

    fn type_check_globals(&self) {
        let globals = (*self.function.dfg.globals).clone();
        for (_, global) in globals.values_iter() {
            let global_typ = global.get_type();
            if global_typ.contains_function() {
                panic!("Globals cannot contain function pointers");
            }
        }
    }

    fn validate_block_terminator(&self, block: BasicBlockId) {
        let terminator = self.function.dfg[block]
            .terminator()
            .expect("Block is expected to have a terminator instruction");

        let entry_block = self.function.entry_block();
        match terminator {
            TerminatorInstruction::JmpIf {
                condition, then_destination, else_destination, ..
            } => {
                let condition_type = self.function.dfg.type_of_value(*condition);
                assert_ne!(
                    *then_destination, entry_block,
                    "Entry block cannot be the target of a jump"
                );
                assert_ne!(
                    *else_destination, entry_block,
                    "Entry block cannot be the target of a jump"
                );
                assert_eq!(
                    condition_type,
                    Type::bool(),
                    "JmpIf conditions should have boolean type"
                );
            }
            TerminatorInstruction::Jmp { destination, .. } => {
                assert_ne!(*destination, entry_block, "Entry block cannot be the target of a jump");
            }
            TerminatorInstruction::Return { return_values, .. } => {
                if let Some(return_data_id) = self.function.dfg.data_bus.return_data {
                    assert_eq!(
                        *return_values,
                        vec![return_data_id],
                        "Databus return_data does not match return terminator"
                    );
                }
            }
            TerminatorInstruction::Unreachable { .. } => (),
        }
    }

    fn run(&mut self) {
        self.type_check_globals();
        self.validate_single_return_block();

        for block in self.function.reachable_blocks() {
            for instruction in self.function.dfg[block].instructions() {
                self.validate_field_to_integer_cast_invariant(*instruction);
                self.type_check_instruction(*instruction);
                self.check_calls_in_unconstrained(*instruction);
                self.check_calls_in_constrained(*instruction);
            }
            self.validate_block_terminator(block);
        }
    }
}

/// Validates that the [Function] is well formed.
///
/// Panics on malformed functions.
pub(crate) fn validate_function(function: &Function, ssa: &Ssa) {
    let mut validator = Validator::new(function, ssa);
    validator.run();
}

fn assert_arguments_length(arguments: &[ValueId], expected: usize, object: &str) {
    let actual = arguments.len();
    assert_eq!(actual, expected, "Expected {object} to have {expected} arguments, got {actual}");
}

fn assert_field(typ: &Type, object: &str) {
    if !matches!(typ, Type::Numeric(NumericType::NativeField)) {
        panic!("{object} must be a Field, not {typ}");
    }
}

fn assert_u1(typ: &Type, object: &str) {
    if !matches!(typ, Type::Numeric(NumericType::Unsigned { bit_size: 1 })) {
        panic!("{object} must be u1, not {typ}");
    }
}

fn assert_u8(typ: &Type, object: &str) {
    if !matches!(typ, Type::Numeric(NumericType::Unsigned { bit_size: 8 })) {
        panic!("{object} must be u8, not {typ}");
    }
}

fn assert_u32(typ: &Type, object: &str) {
    if !matches!(typ, Type::Numeric(NumericType::Unsigned { bit_size: 32 })) {
        panic!("{object} must be u32, not {typ}");
    }
}

fn assert_u64(typ: &Type, object: &str) {
    if !matches!(typ, Type::Numeric(NumericType::Unsigned { bit_size: 64 })) {
        panic!("{object} must be u64, not {typ}");
    }
}

fn assert_vector<'a>(typ: &'a Type, object: &str) -> &'a Arc<Vec<Type>> {
    let Type::Vector(elements) = typ else {
        panic!("{object} must be a vector");
    };
    elements
}

fn assert_array<'a>(typ: &'a Type, object: &str) -> (&'a Arc<Vec<Type>>, u32) {
    let Type::Array(elements, length) = typ else {
        panic!("{object} must be an array");
    };
    (elements, *length)
}

fn assert_u1_array(typ: &Type, object: &str) -> u32 {
    let (elements, length) = assert_array(typ, object);
    assert_eq!(
        elements.len(),
        1,
        "Expected {object} to be an array of length 1, not {}",
        elements.len()
    );
    assert_u1(&elements[0], &format!("{object} array element"));
    length
}

fn assert_u8_array(typ: &Type, object: &str) -> u32 {
    let (elements, length) = assert_array(typ, object);
    assert_eq!(
        elements.len(),
        1,
        "Expected {object} to be an array of length 1, not {}",
        elements.len()
    );
    assert_u8(&elements[0], &format!("{object} array element"));
    length
}

fn assert_u32_array(typ: &Type, object: &str) -> u32 {
    let (elements, length) = assert_array(typ, object);
    assert_eq!(
        elements.len(),
        1,
        "Expected {object} to be an array of length 1, not {}",
        elements.len()
    );
    assert_u32(&elements[0], &format!("{object} array element"));
    length
}

fn assert_u64_array(typ: &Type, object: &str) -> u32 {
    let (elements, length) = assert_array(typ, object);
    assert_eq!(
        elements.len(),
        1,
        "Expected {object} to be an array of length 1, not {}",
        elements.len()
    );
    assert_u64(&elements[0], &format!("{object} array element"));
    length
}

fn assert_field_array(typ: &Type, object: &str) -> u32 {
    let (elements, length) = assert_array(typ, object);
    assert_eq!(
        elements.len(),
        1,
        "Expected {object} to be an array of length 1, not {}",
        elements.len()
    );
    assert_field(&elements[0], &format!("{object} array element"));
    length
}

fn assert_array_length(actual_length: u32, expected_length: u32, object: &str) {
    assert_eq!(
        actual_length, expected_length,
        "Expected {object} to have length {expected_length}, got {actual_length}"
    );
}

fn assert_embedded_curve_point(
    arguments: &[ValueId],
    index: usize,
    dfg: &DataFlowGraph,
    object: &str,
) {
    // struct EmbeddedCurvePoint {
    //     x: Field,
    //     y: Field,
    //     is_infinite: bool,
    // }
    let point_x = arguments[index];
    let point_x_type = dfg.type_of_value(point_x);
    assert_field(&point_x_type, &format!("{object} x"));

    let point_y = arguments[index + 1];
    let point_y_type = dfg.type_of_value(point_y);
    assert_field(&point_y_type, &format!("{object} y"));

    let point_is_infinite = arguments[index + 2];
    let point_is_infinite_type = dfg.type_of_value(point_is_infinite);
    assert_u1(&point_is_infinite_type, &format!("{object} is_infinite"));
}

#[cfg(test)]
mod tests {
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            function::{FunctionId, RuntimeType},
            types::{NumericType, Type},
        },
        ssa_gen::Ssa,
        validation::Validator,
    };

    #[test]
    fn lone_truncate() {
        let src = r"
        acir(inline) pure fn main f0 {
          b0(v0: i16):
            v1 = truncate v0 to 8 bits, max_bit_size: 8
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Cannot use `lt` with field elements")]
    fn disallows_comparing_fields_with_lt() {
        let src = "
        acir(inline) impure fn main f0 {
          b0():
            v2 = lt Field 1, Field 2
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Left-hand side and right-hand side of `add` must have the same type"
    )]
    fn disallows_binary_add_with_different_types() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = add Field 1, i32 2
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Left-hand side and right-hand side of `shr` must have the same type"
    )]
    fn disallows_shr_with_different_type() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = shr u32 1, u16 1
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Left-hand side and right-hand side of `shl` must have the same type"
    )]
    fn disallows_shl_with_different_type() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = shl u32 1, u16 1
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "ToRadix value must be a Field, not u1")]
    fn to_le_radix_on_non_field_value() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0():
            call f1(u1 1)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u1):
            v2 = call to_le_radix(v0, u32 256) -> [u7; 1]
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "ToBits value must be a Field, not u1")]
    fn to_le_bits_on_non_field_value() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0():
            call f1(u1 1)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u1):
            v2 = call to_le_bits(v0) -> [u1; 32]
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn valid_to_le_radix() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = call to_le_radix(v0, u32 256) -> [u8; 1]
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn valid_to_le_bits() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = call to_le_bits(v0) -> [u1; 32]
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[should_panic(
        expected = "Left-hand side and right-hand side of constrain must have the same type"
    )]
    #[test]
    fn constrain_with_different_types() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u8, v1: i8):
            constrain v0 == v1
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[should_panic(
        expected = "Left-hand side and right-hand side of constrain must have the same type"
    )]
    #[test]
    fn constrain_neq_with_different_types() {
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u8, v1: i8):
            constrain v0 != v1
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn cast_from_field_constant_in_range() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = cast Field 42 as u8
            return v0
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn cast_from_field_constant_out_of_range_with_truncate() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = truncate Field 123456 to 8 bits, max_bit_size: 16
            v1 = cast v0 as u8
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn cast_from_field_division_safe() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = div u16 256, u16 256
            v1 = cast v0 as u8
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Constant too large")]
    fn cast_from_field_constant_too_large() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = cast Field 300 as u8
            return v0
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid cast from Field")]
    fn cast_from_raw_field() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = add Field 255, Field 1
            v1 = cast v0 as u8
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "assertion")]
    fn cast_after_unsafe_truncate() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = truncate Field 1000 to 16 bits, max_bit_size: 16
            v1 = cast v0 as u8
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Globals cannot contain function pointers")]
    fn function_pointer_in_global_array() {
        let src = "
        g2 = make_array [f1, f2] : [function; 2]

        acir(inline) fn main f0 {
          b0(v3: u32, v4: Field):
            v6 = call f1() -> Field
            v8 = call f2() -> Field
            v10 = lt v3, u32 2
            constrain v10 == u1 1
            v12 = array_get g2, index v3 -> function
            v13 = call v12() -> Field
            v14 = eq v13, v4
            constrain v13 == v4
            return
        }
        acir(inline) fn f1 f1 {
          b0():
            return Field 1
        }
        acir(inline) fn f2 f2 {
          b0():
            return Field 2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Function call to f1 expected 1 parameters, but got 0")]
    fn call_has_wrong_parameter_count() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }

        acir(inline) fn foo f1 {
          b0(v0: Field):
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Argument #1 to f1 has type Field, but u32 was given")]
    fn call_has_wrong_argument_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            call f1(v0)
            return
        }

        acir(inline) fn foo f1 {
          b0(v0: Field):
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Function call to f1 expected 2 return values, but got 1")]
    fn call_has_wrong_return_count() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0, v1 = call f1() -> (Field, Field)
            return v0
        }

        acir(inline) fn foo f1 {
          b0():
            return Field 1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Function call to f1 expected return type u8, but got Field (at position 1)"
    )]
    fn call_has_wrong_return_type() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = call f1() -> u8
            return v0
        }

        acir(inline) fn foo f1 {
          b0():
            return Field 1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Function f1 has multiple return blocks")]
    fn multiple_return_blocks() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v1 = call f1(u1 1) -> Field
            return v1
        }

        acir(inline) fn f1 f1 {
          b0(v0: u1):
            jmpif v0 then: b1, else: b2
          b1():
            return Field 1
          b2():
            return Field 2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "MakeArray returns an array of flattened length 2, but it has 3 elements"
    )]
    fn make_array_returns_incorrect_length() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [u8 1, u8 2, u8 3] : [u8; 2]
            return v0
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "MakeArray returns an array of flattened length 4, but it has 3 elements"
    )]
    fn make_array_returns_incorrect_length_composite_type() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [u8 1, u8 2, u8 3] : [(u8, u8); 2]
            return v0
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "MakeArray vector has 3 elements but composite type has 2 types which don't divide the number of elements"
    )]
    fn make_array_vector_returns_incorrect_length() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [u8 1, u8 2, u8 3] : [(u8, u8)]
            return v0
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn make_array_vector_empty_composite_type() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [()]
            return v0
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "MakeArray has incorrect element type at index 1: expected u8, got Field"
    )]
    fn make_array_has_incorrect_element_type() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [u8 1, Field 2, u8 3, u8 4] : [(u8, u8); 2]
            return v0
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Store address type u8 does not match value type Field")]
    fn store_has_incorrect_type() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u8
            store Field 1 at v0
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Cannot use `and` with field elements")]
    fn bitwise_and_has_incorrect_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = and v0, v1
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Cannot use `or` with field elements")]
    fn bitwise_or_has_incorrect_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = or v0, v1
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Cannot use `xor` with field elements")]
    fn bitwise_xor_has_incorrect_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = xor v0, v1
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "multi_scalar_mul points is_infinite must be u1, not Field")]
    fn msm_has_incorrect_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [(Field, Field, Field); 3], v1: [(Field, Field); 3], v2: u1):
            v3 = call multi_scalar_mul(v0, v1, v2) -> [(Field, Field, u1); 1]
            return v3
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Call to ACIR function 'foo f1' from unconstrained 'main f0'")]
    fn disallows_calling_acir_from_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            call f1(v0)
            return
        }
        acir(inline) fn foo f1 {
          b0(v0: u32):
            v4 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v5 = array_get v4, index v0 -> Field
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Trying to call foreign function 'oracle_call' from ACIR function 'main f0'"
    )]
    fn disallows_calling_an_oracle_from_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            call oracle_call()
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Trying to pass a reference from ACIR function 'main f0' to unconstrained 'foo f1' in argument v1: &mut u32"
    )]
    fn disallows_passing_refs_from_acir_to_brillig() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut u32
            store v0 at v1
            call f1(v1)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: &mut u32):
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Trying to return a function value to ACIR function 'main f0' from unconstrained 'foo f1' in v2: function"
    )]
    fn disallows_returning_functions_from_brillig_to_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1, v2 = call f1() -> (function, function)
            v3 = call v1(v0) -> u32
            return v3
        }
        brillig(inline) fn foo f1 {
          b0():
            return f2, f3
        }
        acir(inline) fn identity f2 {
          b0(v0: u32):
            return v0
        }
        brillig(inline) fn identity f3 {
          b0(v0: u32):
            return v0
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "JmpIf conditions should have boolean type")]
    fn disallows_non_boolean_jmpif_condition() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            jmpif v0 then: b1, else: b2
          b1():
            jmp b2()
          b2():
            return

        }

        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "AsVector argument must be an array")]
    fn as_vector_length_on_vector_type() {
        let src = "
        acir(inline) fn main f0 {
            b0():
              v3 = make_array [Field 1, Field 2, Field 3] : [Field]
              v4 = call f1(v3) -> u32
              return v4
        }

        acir(inline) fn foo f1 {
            b0(v0: [Field]):
              v2, v3 = call as_vector(v0) -> (u32, [Field])
              return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "AsVector argument must be an array")]
    fn as_vector_length_on_numeric_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2, v3 = call as_vector(v0) -> (u32, [Field])
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "assertion `left == right` failed: Expected AsVector to have 1 arguments, got 0\n  left: 0\n right: 1"
    )]
    fn as_vector_wrong_number_of_arguments() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v1, v2 = call as_vector() -> (u32, [Field])
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "ArrayGet/ArraySet index must be u32")]
    fn array_get_wrong_index_type() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: [u8; 3], v1: u64):
            v2 = array_get v0, index v1 -> u32
            return v2
        }";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "ArrayGet/ArraySet must operate on an array")]
    fn array_get_wrong_array_type() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = array_get v0, index v1 -> u32
            return v2
        }";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn return_data_matches_return_terminator() {
        let src = "
        acir(inline) pure fn main f0 {
          return_data: v4
          b0(v0: u32, v1: u64):
            v2 = cast v0 as Field
            v3 = cast v1 as Field
            v4 = make_array [v2, v3] : [Field; 2]
            return v4
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Databus return_data does not match return terminator")]
    fn return_data_does_not_match_return_terminator() {
        let src = "
        acir(inline) pure fn main f0 {
          return_data: v4
          b0(v0: u32, v1: u64):
            v2 = cast v0 as Field
            v3 = cast v1 as Field
            v4 = make_array [v2, v3] : [Field; 2]
            return v0, v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn allows_return_data_does_with_unreachable_terminator() {
        let src = "
        acir(inline) pure fn main f0 {
          return_data: v4
          b0(v0: u32, v1: u64):
            v4 = make_array [Field 0, Field 0] : [Field; 2]
            unreachable
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Entry block cannot be the target of a jump")]
    fn disallows_jumping_to_entry_block() {
        // This test constructs the following function manually, which cannot be constructed using the SSA parser
        // because the parser does not support jumping to the entry block.
        //
        // brillig(inline) fn main f0 {
        //   b0(v0: u1):
        //     jmp b1()
        //   b1():
        //     jmpif v0 then: b2, else: b3
        //   b2():
        //     jmp b0(Field 0)
        //   b3():
        //     jmp b4()
        //   b4():
        //     return
        // }

        let main_id = FunctionId::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let b0 = builder.current_block();
        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();

        let v0 = builder.add_parameter(Type::bool());

        builder.terminate_with_jmp(b1, Vec::new());

        builder.switch_to_block(b1);

        builder.terminate_with_jmpif(v0, b2, b3);

        builder.switch_to_block(b2);

        let false_constant = builder.numeric_constant(false, NumericType::bool());
        builder.terminate_with_jmp(b0, vec![false_constant]);

        builder.switch_to_block(b3);
        builder.terminate_with_jmp(b4, Vec::new());

        builder.switch_to_block(b4);
        builder.terminate_with_return(Vec::new());

        let ssa = builder.finish();

        Validator::new(&ssa.functions[&main_id], &ssa).run();
    }
}
