#include "schnorr.hpp"
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
}