#include "keccak_constraint.hpp"
#include "barretenberg/stdlib/hash/keccak/keccak.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "round.hpp"

namespace acir_format {

template <typename Builder> void create_keccak_constraints(Builder& builder, const KeccakConstraint& constraint)
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

    byte_array_ct output_bytes = bb::plonk::stdlib::keccak<Builder>::hash(arr);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        builder.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

template <typename Builder> void create_keccak_var_constraints(Builder& builder, const KeccakVarConstraint& constraint)
{
    using byte_array_ct = bb::plonk::stdlib::byte_array<Builder>;
    using field_ct = bb::plonk::stdlib::field_t<Builder>;
    using uint32_ct = bb::plonk::stdlib::uint32<Builder>;

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

    byte_array_ct output_bytes = bb::plonk::stdlib::keccak<Builder>::hash(arr, length);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        builder.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

template <typename Builder> void create_keccak_permutations(Builder& builder, const Keccakf1600& constraint)
{
    using field_ct = bb::plonk::stdlib::field_t<Builder>;

    // Create the array containing the permuted state
    std::array<field_ct, bb::plonk::stdlib::keccak<Builder>::NUM_KECCAK_LANES> state;

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array
    for (size_t i = 0; i < constraint.state.size(); ++i) {
        info(constraint.state[i]);
        state[i] = field_ct::from_witness_index(&builder, constraint.state[i]);
    }

    std::array<field_ct, 25> output_state = bb::plonk::stdlib::keccak<Builder>::permutation_opcode(state, &builder);

    for (size_t i = 0; i < output_state.size(); ++i) {
        builder.assert_equal(output_state[i].normalize().witness_index, constraint.result[i]);
    }
}
template void create_keccak_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                             const KeccakConstraint& constraint);
template void create_keccak_var_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                 const KeccakVarConstraint& constraint);
template void create_keccak_permutations<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                              const Keccakf1600& constraint);

template void create_keccak_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                   const KeccakConstraint& constraint);
template void create_keccak_var_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                       const KeccakVarConstraint& constraint);

template void create_keccak_permutations<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                    const Keccakf1600& constraint);

} // namespace acir_format
