use acvm::acir::{
    AcirField,
    brillig::{BlackBoxOp, IntegerBitSize},
};

use crate::brillig::brillig_ir::BrilligBinaryOp;

use super::{
    BrilligContext,
    brillig_variable::{BrilligArray, SingleAddrVariable},
    debug_show::DebugToString,
    registers::RegisterAllocator,
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

        // If we are truncating a value down to a natively supported integer, we can just use the cast instruction.
        //
        // This is exploiting the fact that the cast instruction will automatically truncate the value when
        // casting to a smaller bit size. We can then avoid the more expensive modulo operation.
        if IntegerBitSize::try_from(bit_size).is_ok() {
            // Allocate a temporary register to hold the intermediate casted value.
            let intermediate_register = self.allocate_single_addr(bit_size);
            self.cast_instruction(*intermediate_register, value_to_truncate);

            // We then cast back to the original type to satisfy typed memory.
            self.cast_instruction(destination_of_truncated_value, *intermediate_register);
            self.deallocate_single_addr(intermediate_register);

            return;
        }

        // If the bit size we are truncating down to is not a natively supported integer, we need to use a modulo operation.

        // The modulus is guaranteed to fit, since we are truncating down to a bit size that is strictly less than the value_to_truncate.bit_size
        let modulus_var = self.make_constant_instruction(
            F::from(2_usize).pow(&F::from(u128::from(bit_size))),
            value_to_truncate.bit_size,
        );

        self.binary_instruction(
            value_to_truncate,
            *modulus_var,
            destination_of_truncated_value,
            BrilligBinaryOp::Modulo,
        );
    }

    /// Issues a `to_radix` instruction. This instruction will write the modulus of the `source_field` register
    /// and the `radix` register `target_array`, with the number of limbs given by the size of `target_array`.
    ///
    /// If `output_bits` is true, it generates bit limbs, otherwise it generates byte limbs.
    /// If `little_endian` is true, then the `target_array` will contain the results in Little Endian order.
    pub(crate) fn codegen_to_radix(
        &mut self,
        source_field: SingleAddrVariable,
        target_array: BrilligArray,
        radix: SingleAddrVariable,
        little_endian: bool,
        output_bits: bool,
    ) {
        assert!(source_field.bit_size == F::max_num_bits());
        assert!(radix.bit_size == 32);

        let bits_register = self.make_constant_instruction(output_bits.into(), 1);
        self.codegen_initialize_array(target_array);
        let pointer = self.codegen_make_array_items_pointer(target_array);
        let num_limbs = self.make_usize_constant_instruction(target_array.size.into());

        // Perform big-endian ToRadix
        self.black_box_op_instruction(BlackBoxOp::ToRadix {
            input: source_field.address,
            radix: radix.address,
            output_pointer: *pointer,
            num_limbs: num_limbs.address,
            output_bits: bits_register.address,
        });

        if little_endian {
            self.codegen_array_reverse(*pointer, num_limbs.address);
        }
    }
}
