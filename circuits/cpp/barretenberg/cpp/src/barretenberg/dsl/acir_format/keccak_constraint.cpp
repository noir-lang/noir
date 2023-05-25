#include "keccak_constraint.hpp"
#include "round.hpp"
#include "barretenberg/stdlib/hash/keccak/keccak.hpp"

namespace acir_format {

void create_keccak_constraints(Composer& composer, const KeccakConstraint& constraint)
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

        arr.write(element_bytes);
    }

    byte_array_ct output_bytes = proof_system::plonk::stdlib::keccak<Composer>::hash(arr);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        composer.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

void create_keccak_var_constraints(Composer& composer, const KeccakVarConstraint& constraint)
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

        arr.write(element_bytes);
    }

    uint32_ct length = field_ct::from_witness_index(&composer, constraint.var_message_size);

    byte_array_ct output_bytes = proof_system::plonk::stdlib::keccak<Composer>::hash(arr, length);

    // Convert byte array to vector of field_t
    auto bytes = output_bytes.bytes();

    for (size_t i = 0; i < bytes.size(); ++i) {
        composer.assert_equal(bytes[i].normalize().witness_index, constraint.result[i]);
    }
}

} // namespace acir_format
