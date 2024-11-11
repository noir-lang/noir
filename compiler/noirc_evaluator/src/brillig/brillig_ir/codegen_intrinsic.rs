use acvm::acir::{
    brillig::{BlackBoxOp, IntegerBitSize},
    AcirField,
};

use crate::brillig::brillig_ir::BrilligBinaryOp;

use super::{
    brillig_variable::{BrilligArray, SingleAddrVariable},
    debug_show::DebugToString,
    registers::RegisterAllocator,
    BrilligContext,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Codegens a truncation of a value to the given bit size
    pub(crate) fn codegen_truncate(
        &mut self,
        destination_of_truncated_value: SingleAddrVariable,
        value_to_truncate: SingleAddrVariable,
        bit_size: u32,
    ) {
        assert!(
            bit_size <= value_to_truncate.bit_size,
            "tried to truncate to a bit size {} greater than the variable size {}",
            bit_size,
            value_to_truncate.bit_size
        );

        if bit_size == value_to_truncate.bit_size {
            self.mov_instruction(destination_of_truncated_value.address, value_to_truncate.address);
            return;
        }

        // If we are truncating a value down to a natively supported integer, we can just use the cast instruction
        if IntegerBitSize::try_from(bit_size).is_ok() {
            // We cast back and forth to ensure that the value is truncated.
            let intermediate_register = SingleAddrVariable::new(self.allocate_register(), bit_size);

            self.cast_instruction(intermediate_register, value_to_truncate);
            self.cast_instruction(destination_of_truncated_value, intermediate_register);

            self.deallocate_single_addr(intermediate_register);
            return;
        }

        // If the bit size we are truncating down to is not a natively supported integer, we need to use a modulo operation.

        // The modulus is guaranteed to fit, since we are truncating down to a bit size that is strictly less than the value_to_truncate.bit_size
        let modulus_var = self.make_constant_instruction(
            F::from(2_usize).pow(&F::from(bit_size as u128)),
            value_to_truncate.bit_size,
        );

        self.binary_instruction(
            value_to_truncate,
            modulus_var,
            destination_of_truncated_value,
            BrilligBinaryOp::Modulo,
        );

        self.deallocate_single_addr(modulus_var);
    }

    /// Issues a to_radix instruction. This instruction will write the modulus of the source register
    /// And the radix register limb_count times to the target vector.
    pub(crate) fn codegen_to_radix(
        &mut self,
        source_field: SingleAddrVariable,
        target_array: BrilligArray,
        radix: SingleAddrVariable,
        little_endian: bool,
        output_bits: bool, // If true will generate bit limbs, if false will generate byte limbs
    ) {
        assert!(source_field.bit_size == F::max_num_bits());
        assert!(radix.bit_size == 32);

        self.codegen_initialize_array(target_array);

        let heap_array = self.codegen_brillig_array_to_heap_array(target_array);

        // Perform big-endian ToRadix
        self.black_box_op_instruction(BlackBoxOp::ToRadix {
            input: source_field.address,
            radix: radix.address,
            output: heap_array,
            output_bits,
        });

        if little_endian {
            let items_len = self.make_usize_constant_instruction(target_array.size.into());
            self.codegen_array_reverse(heap_array.pointer, items_len.address);
            self.deallocate_single_addr(items_len);
        }
        self.deallocate_register(heap_array.pointer);
    }
}
