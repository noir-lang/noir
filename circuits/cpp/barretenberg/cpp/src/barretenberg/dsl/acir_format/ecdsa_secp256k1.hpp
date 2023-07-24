#pragma once
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <vector>

namespace acir_format {

struct EcdsaSecp256k1Constraint {
    // This is the byte representation of the hashed message.
    std::vector<uint32_t> hashed_message;

    // This is the computed signature
    //
    std::vector<uint32_t> signature;

    // This is the supposed public key which signed the
    // message, giving rise to the signature.
    // Since Fr does not have enough bits to represent
    // the prime field in secp256k1, a byte array is used.
    // Can also use low and hi where lo=128 bits
    std::vector<uint32_t> pub_x_indices;
    std::vector<uint32_t> pub_y_indices;

    // This is the result of verifying the signature
    uint32_t result;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(hashed_message, signature, pub_x_indices, pub_y_indices, result);
    friend bool operator==(EcdsaSecp256k1Constraint const& lhs, EcdsaSecp256k1Constraint const& rhs) = default;
};

void create_ecdsa_k1_verify_constraints(Builder& builder,
                                        const EcdsaSecp256k1Constraint& input,
                                        bool has_valid_witness_assignments = true);

void dummy_ecdsa_constraint(Builder& builder, EcdsaSecp256k1Constraint const& input);

crypto::ecdsa::signature ecdsa_convert_signature(Builder& builder, std::vector<uint32_t> signature);
witness_ct ecdsa_index_to_witness(Builder& builder, uint32_t index);
byte_array_ct ecdsa_vector_of_bytes_to_byte_array(Builder& builder, std::vector<uint32_t> vector_of_bytes);

} // namespace acir_format
