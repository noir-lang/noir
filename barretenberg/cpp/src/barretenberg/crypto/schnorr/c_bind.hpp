#include "barretenberg/common/wasm_export.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "multisig.hpp"

extern "C" {

using namespace bb;
using affine_element = grumpkin::g1::affine_element;
using multisig = crypto::schnorr_multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>;

WASM_EXPORT void schnorr_compute_public_key(fr::in_buf private_key, affine_element::out_buf public_key_buf);
WASM_EXPORT void schnorr_negate_public_key(affine_element::in_buf public_key_buffer, affine_element::out_buf output);

WASM_EXPORT void schnorr_construct_signature(uint8_t const* message, fr::in_buf private_key, out_buf32 s, out_buf32 e);

WASM_EXPORT void schnorr_verify_signature(
    uint8_t const* message, affine_element::in_buf pub_key, in_buf32 sig_s, in_buf32 sig_e, bool* result);

WASM_EXPORT void schnorr_multisig_create_multisig_public_key(fq::in_buf private_key,
                                                             multisig::MultiSigPublicKey::out_buf multisig_pubkey_buf);

WASM_EXPORT void schnorr_multisig_validate_and_combine_signer_pubkeys(
    multisig::MultiSigPublicKey::vec_in_buf signer_pubkey_buf, affine_element::out_buf combined_key_buf, bool* success);

WASM_EXPORT void schnorr_multisig_construct_signature_round_1(
    multisig::RoundOnePublicOutput::out_buf round_one_public_output_buf,
    multisig::RoundOnePrivateOutput::out_buf round_one_private_output_buf);

WASM_EXPORT void schnorr_multisig_construct_signature_round_2(
    uint8_t const* message,
    fq::in_buf private_key,
    multisig::RoundOnePrivateOutput::in_buf signer_round_one_private_buf,
    multisig::MultiSigPublicKey::vec_in_buf signer_pubkeys_buf,
    multisig::RoundOnePublicOutput::vec_in_buf round_one_public_buf,
    fq::out_buf round_two_buf,
    bool* success);

WASM_EXPORT void schnorr_multisig_combine_signatures(uint8_t const* message,
                                                     multisig::MultiSigPublicKey::vec_in_buf signer_pubkeys_buf,
                                                     multisig::RoundOnePublicOutput::vec_in_buf round_one_buf,
                                                     fq::vec_in_buf round_two_buf,
                                                     out_buf32 s,
                                                     out_buf32 e,
                                                     bool* success);
}
