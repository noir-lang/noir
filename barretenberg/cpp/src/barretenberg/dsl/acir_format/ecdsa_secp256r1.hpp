#pragma once
#include "barretenberg/dsl/types.hpp"
#include <vector>

namespace acir_format {

struct EcdsaSecp256r1Constraint {
    // This is the byte representation of the hashed message.
    std::vector<uint32_t> hashed_message;

    // This is the supposed public key which signed the
    // message, giving rise to the signature.
    // Since Fr does not have enough bits to represent
    // the prime field in secp256r1, a byte array is used.
    // Can also use low and hi where lo=128 bits
    std::vector<uint32_t> pub_x_indices;
    std::vector<uint32_t> pub_y_indices;

    // This is the result of verifying the signature
    uint32_t result;

    // This is the computed signature
    //
    std::vector<uint32_t> signature;

    friend bool operator==(EcdsaSecp256r1Constraint const& lhs, EcdsaSecp256r1Constraint const& rhs) = default;
};

void create_ecdsa_r1_verify_constraints(Builder& builder,
                                        const EcdsaSecp256r1Constraint& input,
                                        bool has_valid_witness_assignments = true);

void dummy_ecdsa_constraint(Builder& builder, EcdsaSecp256r1Constraint const& input);

template <typename B> inline void read(B& buf, EcdsaSecp256r1Constraint& constraint)
{
    using serialize::read;
    read(buf, constraint.hashed_message);
    read(buf, constraint.signature);
    read(buf, constraint.pub_x_indices);
    read(buf, constraint.pub_y_indices);
    read(buf, constraint.result);
}

template <typename B> inline void write(B& buf, EcdsaSecp256r1Constraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.hashed_message);
    write(buf, constraint.signature);
    write(buf, constraint.pub_x_indices);
    write(buf, constraint.pub_y_indices);
    write(buf, constraint.result);
}

} // namespace acir_format
