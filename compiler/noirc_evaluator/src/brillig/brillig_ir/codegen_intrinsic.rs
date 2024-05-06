use acvm::FieldElement;

use crate::brillig::brillig_ir::BrilligBinaryOp;

use super::{
    brillig_variable::{BrilligVector, SingleAddrVariable},
    BrilligContext,
};

impl BrilligContext {
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

        // We cast back and forth to ensure that the value is truncated.
        let intermediate_register =
            SingleAddrVariable { address: self.allocate_register(), bit_size };
        self.cast_instruction(intermediate_register, value_to_truncate);
        self.cast_instruction(destination_of_truncated_value, intermediate_register);
        self.deallocate_single_addr(intermediate_register);
    }

    /// Issues a to_radix instruction. This instruction will write the modulus of the source register
    /// And the radix register limb_count times to the target vector.
    pub(crate) fn codegen_to_radix(
        &mut self,
        source_field: SingleAddrVariable,
        target_vector: BrilligVector,
        radix: SingleAddrVariable,
        limb_count: SingleAddrVariable,
        big_endian: bool,
        limb_bit_size: u32,
    ) {
        assert!(source_field.bit_size == FieldElement::max_num_bits());
        assert!(radix.bit_size == 32);
        assert!(limb_count.bit_size == 32);
        let radix_as_field =
            SingleAddrVariable::new(self.allocate_register(), FieldElement::max_num_bits());
        self.cast_instruction(radix_as_field, radix);

        self.cast_instruction(SingleAddrVariable::new_usize(target_vector.size), limb_count);
        self.usize_const_instruction(target_vector.rc, 1_usize.into());
        self.codegen_allocate_array(target_vector.pointer, target_vector.size);

        let shifted_field =
            SingleAddrVariable::new(self.allocate_register(), FieldElement::max_num_bits());
        self.mov_instruction(shifted_field.address, source_field.address);

        let limb_field =
            SingleAddrVariable::new(self.allocate_register(), FieldElement::max_num_bits());

        let limb_casted = SingleAddrVariable::new(self.allocate_register(), limb_bit_size);

        self.codegen_loop(target_vector.size, |ctx, iterator_register| {
            // Compute the modulus
            ctx.binary_instruction(
                shifted_field,
                radix_as_field,
                limb_field,
                BrilligBinaryOp::Modulo,
            );
            // Cast it
            ctx.cast_instruction(limb_casted, limb_field);
            // Write it
            ctx.codegen_array_set(target_vector.pointer, iterator_register, limb_casted.address);
            // Integer div the field
            ctx.binary_instruction(
                shifted_field,
                radix_as_field,
                shifted_field,
                BrilligBinaryOp::UnsignedDiv,
            );
        });

        // Deallocate our temporary registers
        self.deallocate_single_addr(shifted_field);
        self.deallocate_single_addr(limb_field);
        self.deallocate_single_addr(limb_casted);
        self.deallocate_single_addr(radix_as_field);

        if big_endian {
            self.codegen_reverse_vector_in_place(target_vector);
        }
    }
}
