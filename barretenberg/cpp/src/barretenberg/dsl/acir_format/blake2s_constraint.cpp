#include "blake2s_constraint.hpp"
#include "barretenberg/stdlib/hash/blake2s/blake2s.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "round.hpp"

namespace acir_format {

using namespace bb;

template <typename Builder> void create_blake2s_constraints(Builder& builder, const Blake2sConstraint& constraint)
{
    using byte_array_ct = stdlib::byte_array<Builder>;
    using field_ct = stdlib::field_t<Builder>;

    // Create byte array struct
    byte_array_ct arr(&builder);

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array
    for (const auto& witness_index_num_bits : constraint.inputs) {
        auto witness_index = witness_index_num_bits.blackbox_input;
        auto num_bits = witness_index_num_bits.num_bits;

        // XXX: The implementation requires us to truncate the element to the nearest byte and not bit
        auto num_bytes = round_to_nearest_byte(num_bits);

        field_ct element = to_field_ct(witness_index, builder);
        byte_array_ct element_bytes(element, num_bytes);

        arr.write(element_bytes);
    }

    byte_array_ct output_bytes = stdlib::blake2s<Builder>(arr);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        builder.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

template void create_blake2s_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                              const Blake2sConstraint& constraint);
template void create_blake2s_constraints<MegaCircuitBuilder>(MegaCircuitBuilder& builder,
                                                             const Blake2sConstraint& constraint);

} // namespace acir_format
