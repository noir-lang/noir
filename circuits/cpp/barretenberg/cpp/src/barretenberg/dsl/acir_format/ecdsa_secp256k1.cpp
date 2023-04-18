#include "ecdsa_secp256k1.hpp"
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp"

namespace acir_format {

using namespace proof_system::plonk;

crypto::ecdsa::signature ecdsa_convert_signature(Composer& composer, std::vector<uint32_t> signature)
{

    crypto::ecdsa::signature signature_cr;

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array

    for (unsigned int i = 0; i < 32; i++) {
        auto witness_index = signature[i];

        std::vector<uint8_t> fr_bytes(sizeof(fr));

        fr value = composer.get_variable(witness_index);

        fr::serialize_to_buffer(value, &fr_bytes[0]);

        signature_cr.r[i] = fr_bytes.back();
    }

    for (unsigned int i = 32; i < 64; i++) {
        auto witness_index = signature[i];

        std::vector<uint8_t> fr_bytes(sizeof(fr));

        fr value = composer.get_variable(witness_index);

        fr::serialize_to_buffer(value, &fr_bytes[0]);

        signature_cr.s[i - 32] = fr_bytes.back();
    }

    return signature_cr;
}

secp256k1_ct::g1_ct ecdsa_convert_inputs(Composer* ctx, const secp256k1::g1::affine_element& input)
{
    uint256_t x_u256(input.x);
    uint256_t y_u256(input.y);
    secp256k1_ct::fq_ct x(witness_ct(ctx, barretenberg::fr(x_u256.slice(0, secp256k1_ct::fq_ct::NUM_LIMB_BITS * 2))),
                          witness_ct(ctx,
                                     barretenberg::fr(x_u256.slice(secp256k1_ct::fq_ct::NUM_LIMB_BITS * 2,
                                                                   secp256k1_ct::fq_ct::NUM_LIMB_BITS * 4))));
    secp256k1_ct::fq_ct y(witness_ct(ctx, barretenberg::fr(y_u256.slice(0, secp256k1_ct::fq_ct::NUM_LIMB_BITS * 2))),
                          witness_ct(ctx,
                                     barretenberg::fr(y_u256.slice(secp256k1_ct::fq_ct::NUM_LIMB_BITS * 2,
                                                                   secp256k1_ct::fq_ct::NUM_LIMB_BITS * 4))));

    return { x, y };
}

// vector of bytes here, assumes that the witness indices point to a field element which can be represented
// with just a byte.
// notice that this function truncates each field_element to a byte
byte_array_ct ecdsa_vector_of_bytes_to_byte_array(Composer& composer, std::vector<uint32_t> vector_of_bytes)
{
    byte_array_ct arr(&composer);

    // Get the witness assignment for each witness index
    // Write the witness assignment to the byte_array
    for (const auto& witness_index : vector_of_bytes) {

        field_ct element = field_ct::from_witness_index(&composer, witness_index);
        size_t num_bytes = 1;

        byte_array_ct element_bytes(element, num_bytes);
        arr.write(element_bytes);
    }
    return arr;
}
witness_ct ecdsa_index_to_witness(Composer& composer, uint32_t index)
{
    fr value = composer.get_variable(index);
    return { &composer, value };
}

void create_ecdsa_verify_constraints(Composer& composer, const EcdsaSecp256k1Constraint& input)
{

    auto new_sig = ecdsa_convert_signature(composer, input.signature);

    auto message = ecdsa_vector_of_bytes_to_byte_array(composer, input.message);
    auto pub_key_x_byte_arr = ecdsa_vector_of_bytes_to_byte_array(composer, input.pub_x_indices);
    auto pub_key_y_byte_arr = ecdsa_vector_of_bytes_to_byte_array(composer, input.pub_y_indices);

    auto pub_key_x_fq = secp256k1_ct::fq_ct(pub_key_x_byte_arr);
    auto pub_key_y_fq = secp256k1_ct::fq_ct(pub_key_y_byte_arr);

    std::vector<uint8_t> rr(new_sig.r.begin(), new_sig.r.end());
    std::vector<uint8_t> ss(new_sig.s.begin(), new_sig.s.end());

    stdlib::ecdsa::signature<Composer> sig{ stdlib::byte_array<Composer>(&composer, rr),
                                            stdlib::byte_array<Composer>(&composer, ss) };

    pub_key_x_fq.assert_is_in_field();
    pub_key_y_fq.assert_is_in_field();

    secp256k1_ct::g1_bigfr_ct public_key = secp256k1_ct::g1_bigfr_ct(pub_key_x_fq, pub_key_y_fq);

    bool_ct signature_result = stdlib::ecdsa::verify_signature<Composer,
                                                               secp256k1_ct,
                                                               secp256k1_ct::fq_ct,
                                                               secp256k1_ct::bigfr_ct,
                                                               secp256k1_ct::g1_bigfr_ct>(message, public_key, sig);

    bool_ct signature_result_normalized = signature_result.normalize();

    composer.assert_equal(signature_result_normalized.witness_index, input.result);
}

} // namespace acir_format
