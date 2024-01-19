#include "sha256_constraint.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "round.hpp"

namespace acir_format {

// This function does not work (properly) because the stdlib:sha256 function is not working correctly for 512 bits
// pair<witness_index, bits>
template <typename Builder> void create_sha256_constraints(Builder& builder, const Sha256Constraint& constraint)
{
    using byte_array_ct = bb::plonk::stdlib::byte_array<Builder>;
    using field_ct = bb::plonk::stdlib::field_t<Builder>;

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

        arr.write(element_bytes);
    }

    // Compute sha256
    byte_array_ct output_bytes = bb::plonk::stdlib::sha256<Builder>(arr);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        builder.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

template void create_sha256_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                             const Sha256Constraint& constraint);
template void create_sha256_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                   const Sha256Constraint& constraint);

} // namespace acir_format
