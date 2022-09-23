#include "schnorr.hpp"
#include "multisig.hpp"

#include <ecc/curves/grumpkin/grumpkin.hpp>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void compute_public_key(uint8_t const* private_key, uint8_t* public_key_buf)
{
    auto priv_key = from_buffer<grumpkin::fr>(private_key);
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    write(public_key_buf, pub_key);
}

WASM_EXPORT void construct_signature(
    uint8_t const* message, size_t msg_len, uint8_t const* private_key, uint8_t* s, uint8_t* e)
{
    auto priv_key = from_buffer<grumpkin::fr>(private_key);
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> key_pair = { priv_key, pub_key };
    auto sig = crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq>(std::string((char*)message, msg_len),
                                                                                 key_pair);
    write(s, sig.s);
    write(e, sig.e);
}

WASM_EXPORT bool verify_signature(
    uint8_t const* message, size_t msg_len, uint8_t const* pub_key, uint8_t const* sig_s, uint8_t const* sig_e)
{
    auto pubk = from_buffer<grumpkin::g1::affine_element>(pub_key);
    std::array<uint8_t, 32> s, e;
    std::copy(sig_s, sig_s + 32, s.begin());
    std::copy(sig_e, sig_e + 32, e.begin());
    crypto::schnorr::signature sig = { s, e };
    return crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        std::string((char*)message, msg_len), pubk, sig);
}

WASM_EXPORT void multisig_create_multisig_public_key(uint8_t const* private_key, uint8_t* multisig_pubkey_buf)
{
    using multisig = crypto::schnorr::multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>;
    using multisig_public_key = typename multisig::MultiSigPublicKey;
    auto priv_key = from_buffer<grumpkin::fr>(private_key);
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> key_pair = { priv_key, pub_key };

    auto agg_pubkey = multisig_public_key(key_pair);

    write(multisig_pubkey_buf, agg_pubkey);
}

WASM_EXPORT bool multisig_validate_and_combine_signer_pubkeys(uint8_t const* signer_pubkey_buf,
                                                              uint8_t* combined_key_buf)
{
    using multisig = crypto::schnorr::multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>;
    std::vector<multisig::MultiSigPublicKey> pubkeys =
        from_buffer<std::vector<multisig::MultiSigPublicKey>>(signer_pubkey_buf);

    if (auto combined_key = multisig::validate_and_combine_signer_pubkeys(pubkeys)) {
        write(combined_key_buf, *combined_key);
        return true;
    } else {
        return false;
    }
}

WASM_EXPORT void multisig_construct_signature_round_1(uint8_t* round_one_public_output_buf,
                                                      uint8_t* round_one_private_output_buf)
{
    using multisig = crypto::schnorr::multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>;

    auto [public_output, private_output] = multisig::construct_signature_round_1();
    write(round_one_public_output_buf, public_output);
    write(round_one_private_output_buf, private_output);
}

WASM_EXPORT bool multisig_construct_signature_round_2(uint8_t const* message,
                                                      size_t msg_len,
                                                      uint8_t* const private_key,
                                                      uint8_t* const signer_round_one_private_buf,
                                                      uint8_t* const signer_pubkeys_buf,
                                                      uint8_t* const round_one_public_buf,
                                                      uint8_t* round_two_buf)
{
    using multisig = crypto::schnorr::multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>;
    auto priv_key = from_buffer<grumpkin::fr>(private_key);
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> key_pair = { priv_key, pub_key };

    auto signer_pubkeys = from_buffer<std::vector<multisig::MultiSigPublicKey>>(signer_pubkeys_buf);
    auto round_one_outputs = from_buffer<std::vector<multisig::RoundOnePublicOutput>>(round_one_public_buf);

    auto round_one_private = from_buffer<multisig::RoundOnePrivateOutput>(signer_round_one_private_buf);
    auto round_two_output = multisig::construct_signature_round_2(
        std::string((char*)message, msg_len), key_pair, round_one_private, signer_pubkeys, round_one_outputs);

    if (round_two_output.has_value()) {
        write(round_two_buf, *round_two_output);
        return true;
    } else {
        return false;
    }
}

WASM_EXPORT bool multisig_combine_signatures(uint8_t const* message,
                                             size_t msg_len,
                                             uint8_t* const signer_pubkeys_buf,
                                             uint8_t* const round_one_buf,
                                             uint8_t* const round_two_buf,
                                             uint8_t* s,
                                             uint8_t* e)
{
    using multisig = crypto::schnorr::multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>;

    auto signer_pubkeys = from_buffer<std::vector<multisig::MultiSigPublicKey>>(signer_pubkeys_buf);
    auto round_one_outputs = from_buffer<std::vector<multisig::RoundOnePublicOutput>>(round_one_buf);
    auto round_two_outputs = from_buffer<std::vector<multisig::RoundTwoPublicOutput>>(round_two_buf);

    auto sig = multisig::combine_signatures(
        std::string((char*)message, msg_len), signer_pubkeys, round_one_outputs, round_two_outputs);

    if (sig.has_value()) {
        write(s, (*sig).s);
        write(e, (*sig).e);
        return true;
    } else {
        return false;
    }
}
}