#include "sha256_constraint.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256_plookup.hpp"
#include "round.hpp"

namespace acir_format {

// This function does not work (properly) because the stdlib:sha256 function is not working correctly for 512 bits
// pair<witness_index, bits>
template <typename Builder> void create_sha256_constraints(Builder& builder, const Sha256Constraint& constraint)
{
    using byte_array_ct = bb::stdlib::byte_array<Builder>;
    using field_ct = bb::stdlib::field_t<Builder>;

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
    byte_array_ct output_bytes = bb::stdlib::sha256<Builder>(arr);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        builder.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

template <typename Builder>
void create_sha256_compression_constraints(Builder& builder, const Sha256Compression& constraint)
{
    using field_ct = bb::stdlib::field_t<Builder>;

    std::array<field_ct, 16> inputs;
    std::array<field_ct, 8> hash_inputs;

    // Get the witness assignment for each witness index
    // Note that we do not range-check the inputs, which should be 32 bits,
    // because of the lookup-tables.
    size_t i = 0;
    for (const auto& witness_index_num_bits : constraint.inputs) {
        auto witness_index = witness_index_num_bits.witness;
        field_ct element = field_ct::from_witness_index(&builder, witness_index);
        inputs[i] = element;
        ++i;
    }
    i = 0;
    for (const auto& witness_index_num_bits : constraint.hash_values) {
        auto witness_index = witness_index_num_bits.witness;
        field_ct element = field_ct::from_witness_index(&builder, witness_index);
        hash_inputs[i] = element;
        ++i;
    }

    // Compute sha256 compression
    auto output_bytes = bb::stdlib::sha256_plookup::sha256_block<Builder>(hash_inputs, inputs);

    for (size_t i = 0; i < 8; ++i) {
        poly_triple assert_equal{
            .a = output_bytes[i].normalize().witness_index,
            .b = constraint.result[i],
            .c = 0,
            .q_m = 0,
            .q_l = 1,
            .q_r = -1,
            .q_o = 0,
            .q_c = 0,
        };
        builder.create_poly_gate(assert_equal);
    }
}

template void create_sha256_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                             const Sha256Constraint& constraint);
template void create_sha256_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                   const Sha256Constraint& constraint);

template void create_sha256_compression_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                         const Sha256Compression& constraint);
template void create_sha256_compression_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                               const Sha256Compression& constraint);

} // namespace acir_format
