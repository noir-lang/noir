#include "hash_to_field.hpp"
#include "round.hpp"
#include "barretenberg/stdlib/types/types.hpp"

using namespace plonk::stdlib::types;

namespace acir_format {

void create_hash_to_field_constraints(plonk::TurboComposer& composer, const HashToFieldConstraint constraint)
{

    // Create byte array struct
    byte_array_ct arr(&composer);

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array
    for (const auto& witness_index_num_bits : constraint.inputs) {
        auto witness_index = witness_index_num_bits.witness;
        auto num_bits = witness_index_num_bits.num_bits;

        // XXX: The implementation requires us to truncate the element to the nearest byte and not bit
        auto num_bytes = round_to_nearest_byte(num_bits);

        field_ct element = field_ct::from_witness_index(&composer, witness_index);
        byte_array_ct element_bytes(element, num_bytes);
        byte_array_ct reversed_bytes = element_bytes.reverse();

        arr.write(reversed_bytes);
    }

    // Hash To Field using blake2s.
    // Note: It does not need to be blake2s in the future

    byte_array_ct out_bytes = plonk::stdlib::blake2s<plonk::TurboComposer>(arr);

    field_ct out(out_bytes);
    field_ct normalised_out = out.normalize();

    composer.assert_equal(normalised_out.witness_index, constraint.result);
}

} // namespace acir_format
