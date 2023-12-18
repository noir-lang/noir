#include "keccak_constraint.hpp"
#include "barretenberg/stdlib/hash/keccak/keccak.hpp"
#include "round.hpp"

namespace acir_format {

template <typename Builder> void create_keccak_constraints(Builder& builder, const KeccakConstraint& constraint)
{
    using byte_array_ct = proof_system::plonk::stdlib::byte_array<Builder>;
    using field_ct = proof_system::plonk::stdlib::field_t<Builder>;

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

    byte_array_ct output_bytes = proof_system::plonk::stdlib::keccak<Builder>::hash(arr);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        builder.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

template <typename Builder> void create_keccak_var_constraints(Builder& builder, const KeccakVarConstraint& constraint)
{
    using byte_array_ct = proof_system::plonk::stdlib::byte_array<Builder>;
    using field_ct = proof_system::plonk::stdlib::field_t<Builder>;
    using uint32_ct = proof_system::plonk::stdlib::uint32<Builder>;

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

    uint32_ct length = field_ct::from_witness_index(&builder, constraint.var_message_size);

    byte_array_ct output_bytes = proof_system::plonk::stdlib::keccak<Builder>::hash(arr, length);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        builder.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

template void create_keccak_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                             const KeccakConstraint& constraint);
template void create_keccak_var_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                 const KeccakVarConstraint& constraint);
template void create_keccak_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                   const KeccakConstraint& constraint);
template void create_keccak_var_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                       const KeccakVarConstraint& constraint);

} // namespace acir_format
