use acvm::acir::{
    brillig::{BlackBoxOp, HeapArray, IntegerBitSize},
    AcirField,
};

use crate::brillig::brillig_ir::BrilligBinaryOp;

use super::{
    brillig_variable::{BrilligVector, SingleAddrVariable},
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
        target_vector: BrilligVector,
        radix: u32,
        limb_count: usize,
        big_endian: bool,
        limb_bit_size: u32,
    ) {
        assert!(source_field.bit_size == F::max_num_bits());

        self.usize_const_instruction(target_vector.size, limb_count.into());
        self.usize_const_instruction(target_vector.rc, 1_usize.into());
        self.codegen_allocate_array(target_vector.pointer, target_vector.size);

        self.black_box_op_instruction(BlackBoxOp::ToRadix {
            input: source_field.address,
            radix,
            output: HeapArray { pointer: target_vector.pointer, size: limb_count },
        });

        if limb_bit_size != F::max_num_bits() {
            let end_pointer = self.allocate_register();
            let temporary_register = self.allocate_register();

            self.memory_op_instruction(
                target_vector.pointer,
                target_vector.size,
                end_pointer,
                BrilligBinaryOp::Add,
            );

            self.codegen_for_loop(
                Some(target_vector.pointer),
                end_pointer,
                None,
                |ctx, item_pointer| {
                    ctx.load_instruction(temporary_register, item_pointer.address);

                    ctx.cast(
                        SingleAddrVariable::new(temporary_register, limb_bit_size),
                        SingleAddrVariable::new(temporary_register, F::max_num_bits()),
                    );

                    ctx.store_instruction(item_pointer.address, temporary_register);
                },
            );

            self.deallocate_register(end_pointer);
            self.deallocate_register(temporary_register);
        }

        if big_endian {
            self.codegen_array_reverse(target_vector.pointer, target_vector.size);
        }
    }
}
