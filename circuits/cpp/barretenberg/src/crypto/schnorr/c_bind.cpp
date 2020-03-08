#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "schnorr.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void compute_public_key(uint8_t const* private_key, uint8_t* public_key_buf)
{
    auto priv_key = grumpkin::fr::serialize_from_buffer(private_key);
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    grumpkin::g1::affine_element::serialize_to_buffer(pub_key, public_key_buf);
}

WASM_EXPORT void construct_signature(
    uint8_t const* message, size_t msg_len, uint8_t const* private_key, uint8_t* s, uint8_t* e)
{
    auto priv_key = grumpkin::fr::serialize_from_buffer(private_key);
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> key_pair = { priv_key, pub_key };
    auto sig = crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq>(std::string((char*)message, msg_len),
                                                                                 key_pair);
    std::copy(sig.s.begin(), sig.s.end(), s);
    std::copy(sig.e.begin(), sig.e.end(), e);
}

WASM_EXPORT bool verify_signature(
    uint8_t const* message, size_t msg_len, uint8_t const* pub_key, uint8_t const* sig_s, uint8_t const* sig_e)
{
    auto pubk = grumpkin::g1::affine_element::serialize_from_buffer(const_cast<uint8_t*>(pub_key));
    std::array<uint8_t, 32> s, e;
    std::copy(sig_s, sig_s + 32, s.begin());
    std::copy(sig_e, sig_e + 32, e.begin());
    crypto::schnorr::signature sig = { s, e };
    return crypto::schnorr::verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        std::string((char*)message, msg_len), pubk, sig);
}
}