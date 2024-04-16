#pragma once
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <vector>

namespace acir_format {

struct EcdsaSecp256k1Constraint {
    // This is the byte representation of the hashed message.
    std::array<uint32_t, 32> hashed_message;

    // This is the computed signature
    //
    std::array<uint32_t, 64> signature;

    // This is the supposed public key which signed the
    // message, giving rise to the signature.
    // Since Fr does not have enough bits to represent
    // the prime field in secp256k1, a byte array is used.
    // Can also use low and hi where lo=128 bits
    std::array<uint32_t, 32> pub_x_indices;
    std::array<uint32_t, 32> pub_y_indices;

    // This is the result of verifying the signature
    uint32_t result;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(hashed_message, signature, pub_x_indices, pub_y_indices, result);
    friend bool operator==(EcdsaSecp256k1Constraint const& lhs, EcdsaSecp256k1Constraint const& rhs) = default;
};

template <typename Builder>
void create_ecdsa_k1_verify_constraints(Builder& builder,
                                        const EcdsaSecp256k1Constraint& input,
                                        bool has_valid_witness_assignments = true);

template <typename Builder> void dummy_ecdsa_constraint(Builder& builder, EcdsaSecp256k1Constraint const& input);

witness_ct ecdsa_index_to_witness(Builder& builder, uint32_t index);
template <std::size_t SIZE, typename Builder>
bb::stdlib::byte_array<Builder> ecdsa_array_of_bytes_to_byte_array(Builder& builder,
                                                                   std::array<uint32_t, SIZE> vector_of_bytes);

// We have the implementation of this template in the header as this method is used
// by other ecdsa constraints over different curves (e.g. secp256r1).
// gcc needs to be able to see the implementation order to generate code for
// all Builder specializations (e.g. bb::Goblin::Builder vs. bb::UltraCircuitBuilder)
template <typename Builder>
crypto::ecdsa_signature ecdsa_convert_signature(Builder& builder, std::array<uint32_t, 64> signature)
{

    crypto::ecdsa_signature signature_cr;

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array

    for (unsigned int i = 0; i < 32; i++) {
        auto witness_index = signature[i];

        std::vector<uint8_t> fr_bytes(sizeof(fr));

        fr value = builder.get_variable(witness_index);

        fr::serialize_to_buffer(value, &fr_bytes[0]);

        signature_cr.r[i] = fr_bytes.back();
    }

    for (unsigned int i = 32; i < 64; i++) {
        auto witness_index = signature[i];

        std::vector<uint8_t> fr_bytes(sizeof(fr));

        fr value = builder.get_variable(witness_index);

        fr::serialize_to_buffer(value, &fr_bytes[0]);

        signature_cr.s[i - 32] = fr_bytes.back();
    }

    signature_cr.v = 27;

    return signature_cr;
}

} // namespace acir_format
