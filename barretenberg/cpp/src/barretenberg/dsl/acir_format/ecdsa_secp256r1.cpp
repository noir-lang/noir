#include "ecdsa_secp256r1.hpp"
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp"
#include "ecdsa_secp256k1.hpp"

namespace acir_format {

using namespace bb::plonk;

secp256r1_ct::g1_ct ecdsa_convert_inputs(Builder* ctx, const secp256r1::g1::affine_element& input)
{
    uint256_t x_u256(input.x);
    uint256_t y_u256(input.y);
    secp256r1_ct::fq_ct x(
        witness_ct(ctx, bb::fr(x_u256.slice(0, secp256r1_ct::fq_ct::NUM_LIMB_BITS * 2))),
        witness_ct(
            ctx, bb::fr(x_u256.slice(secp256r1_ct::fq_ct::NUM_LIMB_BITS * 2, secp256r1_ct::fq_ct::NUM_LIMB_BITS * 4))));
    secp256r1_ct::fq_ct y(
        witness_ct(ctx, bb::fr(y_u256.slice(0, secp256r1_ct::fq_ct::NUM_LIMB_BITS * 2))),
        witness_ct(
            ctx, bb::fr(y_u256.slice(secp256r1_ct::fq_ct::NUM_LIMB_BITS * 2, secp256r1_ct::fq_ct::NUM_LIMB_BITS * 4))));

    return { x, y };
}

template <typename Builder>
void create_ecdsa_r1_verify_constraints(Builder& builder,
                                        const EcdsaSecp256r1Constraint& input,
                                        bool has_valid_witness_assignments)
{
    using secp256r1_ct = bb::stdlib::secp256r1<Builder>;
    using bool_ct = bb::stdlib::bool_t<Builder>;
    using field_ct = bb::stdlib::field_t<Builder>;

    if (has_valid_witness_assignments == false) {
        dummy_ecdsa_constraint(builder, input);
    }

    auto new_sig = ecdsa_convert_signature(builder, input.signature);

    auto message = ecdsa_vector_of_bytes_to_byte_array(builder, input.hashed_message);
    auto pub_key_x_byte_arr = ecdsa_vector_of_bytes_to_byte_array(builder, input.pub_x_indices);
    auto pub_key_y_byte_arr = ecdsa_vector_of_bytes_to_byte_array(builder, input.pub_y_indices);

    auto pub_key_x_fq = typename secp256r1_ct::fq_ct(pub_key_x_byte_arr);
    auto pub_key_y_fq = typename secp256r1_ct::fq_ct(pub_key_y_byte_arr);

    std::vector<uint8_t> rr(new_sig.r.begin(), new_sig.r.end());
    std::vector<uint8_t> ss(new_sig.s.begin(), new_sig.s.end());
    uint8_t vv = new_sig.v;

    stdlib::ecdsa::signature<Builder> sig{ stdlib::byte_array<Builder>(&builder, rr),
                                           stdlib::byte_array<Builder>(&builder, ss),
                                           stdlib::uint8<Builder>(&builder, vv) };

    pub_key_x_fq.assert_is_in_field();
    pub_key_y_fq.assert_is_in_field();
    typename secp256r1_ct::g1_bigfr_ct public_key = typename secp256r1_ct::g1_bigfr_ct(pub_key_x_fq, pub_key_y_fq);
    for (size_t i = 0; i < 32; ++i) {
        sig.r[i].assert_equal(field_ct::from_witness_index(&builder, input.signature[i]));
        sig.s[i].assert_equal(field_ct::from_witness_index(&builder, input.signature[i + 32]));
        pub_key_x_byte_arr[i].assert_equal(field_ct::from_witness_index(&builder, input.pub_x_indices[i]));
        pub_key_y_byte_arr[i].assert_equal(field_ct::from_witness_index(&builder, input.pub_y_indices[i]));
    }
    for (size_t i = 0; i < input.hashed_message.size(); ++i) {
        message[i].assert_equal(field_ct::from_witness_index(&builder, input.hashed_message[i]));
    }

    bool_ct signature_result =
        stdlib::ecdsa::verify_signature_prehashed_message_noassert<Builder,
                                                                   secp256r1_ct,
                                                                   typename secp256r1_ct::fq_ct,
                                                                   typename secp256r1_ct::bigfr_ct,
                                                                   typename secp256r1_ct::g1_bigfr_ct>(
            message, public_key, sig);
    bool_ct signature_result_normalized = signature_result.normalize();
    builder.assert_equal(signature_result_normalized.witness_index, input.result);
}

// Add dummy constraints for ECDSA because when the verifier creates the
// constraint system, they usually use zeroes for witness values.
//
// This does not work for ECDSA as the signature, r, s and public key need
// to be valid.
template <typename Builder> void dummy_ecdsa_constraint(Builder& builder, EcdsaSecp256r1Constraint const& input)
{

    std::vector<uint32_t> pub_x_indices_;
    std::vector<uint32_t> pub_y_indices_;
    std::vector<uint32_t> signature_;
    std::vector<uint32_t> message_indices_;
    signature_.resize(64);

    // Create a valid signature with a valid public key
    std::string message_string = "Instructions unclear, ask again later.";

    // hash the message since the dsl ecdsa gadget uses the prehashed message
    // NOTE: If the hash being used outputs more than 32 bytes, then big-field will panic
    std::vector<uint8_t> message_buffer;
    std::copy(message_string.begin(), message_string.end(), std::back_inserter(message_buffer));
    auto hashed_message = sha256::sha256(message_buffer);

    crypto::ecdsa::key_pair<secp256r1::fr, secp256r1::g1> account;
    account.private_key = 10;
    account.public_key = secp256r1::g1::one * account.private_key;

    crypto::ecdsa::signature signature =
        crypto::ecdsa::construct_signature<Sha256Hasher, secp256r1::fq, secp256r1::fr, secp256r1::g1>(message_string,
                                                                                                      account);

    uint256_t pub_x_value = account.public_key.x;
    uint256_t pub_y_value = account.public_key.y;

    // Create new variables which will reference the valid public key and signature.
    // We don't use them in a gate, so when we call assert_equal, they will be
    // replaced as if they never existed.
    for (size_t i = 0; i < 32; ++i) {
        uint32_t m_wit = builder.add_variable(hashed_message[i]);
        uint32_t x_wit = builder.add_variable(pub_x_value.slice(248 - i * 8, 256 - i * 8));
        uint32_t y_wit = builder.add_variable(pub_y_value.slice(248 - i * 8, 256 - i * 8));
        uint32_t r_wit = builder.add_variable(signature.r[i]);
        uint32_t s_wit = builder.add_variable(signature.s[i]);
        message_indices_.emplace_back(m_wit);
        pub_x_indices_.emplace_back(x_wit);
        pub_y_indices_.emplace_back(y_wit);
        signature_[i] = r_wit;
        signature_[i + 32] = s_wit;
    }

    // Call assert_equal(from, to) to replace the value in `to` by the value in `from`
    for (size_t i = 0; i < input.hashed_message.size(); ++i) {
        builder.assert_equal(message_indices_[i], input.hashed_message[i]);
    }
    for (size_t i = 0; i < input.pub_x_indices.size(); ++i) {
        builder.assert_equal(pub_x_indices_[i], input.pub_x_indices[i]);
    }
    for (size_t i = 0; i < input.pub_y_indices.size(); ++i) {
        builder.assert_equal(pub_y_indices_[i], input.pub_y_indices[i]);
    }
    for (size_t i = 0; i < input.signature.size(); ++i) {
        builder.assert_equal(signature_[i], input.signature[i]);
    }
}

template void create_ecdsa_r1_verify_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                      const EcdsaSecp256r1Constraint& input,
                                                                      bool has_valid_witness_assignments);
template void create_ecdsa_r1_verify_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                            const EcdsaSecp256r1Constraint& input,
                                                                            bool has_valid_witness_assignments);
template void dummy_ecdsa_constraint<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                          EcdsaSecp256r1Constraint const& input);

} // namespace acir_format
