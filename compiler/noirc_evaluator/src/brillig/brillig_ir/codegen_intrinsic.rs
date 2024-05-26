use acvm::{
    acir::brillig::{BlackBoxOp, HeapArray},
    acir::AcirField,
    FieldElement,
};

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
        radix: u32,
        limb_count: usize,
        big_endian: bool,
        limb_bit_size: u32,
    ) {
        assert!(source_field.bit_size == FieldElement::max_num_bits());

        self.usize_const_instruction(target_vector.size, limb_count.into());
        self.usize_const_instruction(target_vector.rc, 1_usize.into());
        self.codegen_allocate_array(target_vector.pointer, target_vector.size);

        self.black_box_op_instruction(BlackBoxOp::ToRadix {
            input: source_field.address,
            radix,
            output: HeapArray { pointer: target_vector.pointer, size: limb_count },
        });

        let limb_field =
            SingleAddrVariable::new(self.allocate_register(), FieldElement::max_num_bits());

        let limb_casted = SingleAddrVariable::new(self.allocate_register(), limb_bit_size);

        if limb_bit_size != FieldElement::max_num_bits() {
            self.codegen_loop(target_vector.size, |ctx, iterator_register| {
                // Read the limb
                ctx.codegen_array_get(target_vector.pointer, iterator_register, limb_field.address);
                // Cast it
                ctx.cast_instruction(limb_casted, limb_field);
                // Write it
                ctx.codegen_array_set(
                    target_vector.pointer,
                    iterator_register,
                    limb_casted.address,
                );
            });
        }

        // Deallocate our temporary registers
        self.deallocate_single_addr(limb_field);
        self.deallocate_single_addr(limb_casted);

        if big_endian {
            self.codegen_reverse_vector_in_place(target_vector);
        }
    }
}
