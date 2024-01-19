#include "schnorr_verify.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"
#include "barretenberg/stdlib/encryption/schnorr/schnorr.hpp"

namespace acir_format {

using namespace bb::stdlib;

template <typename Builder>
crypto::schnorr::signature convert_signature(Builder& builder, std::vector<uint32_t> signature)
{

    crypto::schnorr::signature signature_cr;

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array

    for (unsigned int i = 0; i < 32; i++) {
        auto witness_index = signature[i];

        std::vector<uint8_t> fr_bytes(sizeof(fr));

        fr value = builder.get_variable(witness_index);

        fr::serialize_to_buffer(value, &fr_bytes[0]);

        signature_cr.s[i] = fr_bytes.back();
    }

    for (unsigned int i = 32; i < 64; i++) {
        auto witness_index = signature[i];

        std::vector<uint8_t> fr_bytes(sizeof(fr));

        fr value = builder.get_variable(witness_index);

        fr::serialize_to_buffer(value, &fr_bytes[0]);

        signature_cr.e[i - 32] = fr_bytes.back();
    }

    return signature_cr;
}
// vector of bytes here, assumes that the witness indices point to a field element which can be represented
// with just a byte.
// notice that this function truncates each field_element to a byte
template <typename Builder>
bb::stdlib::byte_array<Builder> vector_of_bytes_to_byte_array(Builder& builder, std::vector<uint32_t> vector_of_bytes)
{
    using byte_array_ct = bb::stdlib::byte_array<Builder>;
    using field_ct = bb::stdlib::field_t<Builder>;

    byte_array_ct arr(&builder);

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array
    for (const auto& witness_index : vector_of_bytes) {

        field_ct element = field_ct::from_witness_index(&builder, witness_index);
        size_t num_bytes = 1;

        byte_array_ct element_bytes(element, num_bytes);
        arr.write(element_bytes);
    }
    return arr;
}

template <typename Builder> bb::stdlib::witness_t<Builder> index_to_witness(Builder& builder, uint32_t index)
{
    fr value = builder.get_variable(index);
    return { &builder, value };
}

template <typename Builder> void create_schnorr_verify_constraints(Builder& builder, const SchnorrConstraint& input)
{
    using witness_ct = bb::stdlib::witness_t<Builder>;
    using cycle_group_ct = bb::stdlib::cycle_group<Builder>;
    using schnorr_signature_bits_ct = bb::stdlib::schnorr::signature_bits<Builder>;
    using bool_ct = bb::stdlib::bool_t<Builder>;

    auto new_sig = convert_signature(builder, input.signature);
    // From ignorance, you will see me convert a bunch of witnesses from ByteArray -> BitArray
    // This may not be the most efficient way to do it. It is being used as it is known to work,
    // optimizations are welcome!

    // First convert the message of u8 witnesses into a byte_array
    // Do this by taking each element as a u8 and writing it to the byte array

    auto message = vector_of_bytes_to_byte_array(builder, input.message);

    fr pubkey_value_x = builder.get_variable(input.public_key_x);
    fr pubkey_value_y = builder.get_variable(input.public_key_y);

    cycle_group_ct pub_key{ witness_ct(&builder, pubkey_value_x), witness_ct(&builder, pubkey_value_y), false };

    schnorr_signature_bits_ct sig = schnorr::convert_signature(&builder, new_sig);

    bool_ct signature_result = schnorr::signature_verification_result(message, pub_key, sig);

    bool_ct signature_result_normalized = signature_result.normalize();

    builder.assert_equal(signature_result_normalized.witness_index, input.result);
}

template void create_schnorr_verify_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                     const SchnorrConstraint& input);
template void create_schnorr_verify_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                           const SchnorrConstraint& input);

} // namespace acir_format
