#include "hash_to_field.hpp"
#include "round.hpp"

namespace acir_format {

using namespace proof_system::plonk;

void create_hash_to_field_constraints(Builder& builder, const HashToFieldConstraint constraint)
{

    // Create byte array struct
    byte_array_ct arr(&builder);

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array
    for (const auto& witness_index_num_bits : constraint.inputs) {
        auto witness_index = witness_index_num_bits.witness;
        auto num_bits = witness_index_num_bits.num_bits;

        // XXX: The implementation requires us to truncate the element to the nearest byte and not bit
        auto num_bytes = round_to_nearest_byte(num_bits);

        field_ct element = field_ct::from_witness_index(&builder, witness_index);
        byte_array_ct element_bytes(element, num_bytes);
        byte_array_ct reversed_bytes = element_bytes.reverse();

        arr.write(reversed_bytes);
    }

    // Hash To Field using blake2s.
    // Note: It does not need to be blake2s in the future

    byte_array_ct out_bytes = stdlib::blake2s<Builder>(arr);

    field_ct out(out_bytes);
    field_ct normalised_out = out.normalize();

    builder.assert_equal(normalised_out.witness_index, constraint.result);
}

} // namespace acir_format
