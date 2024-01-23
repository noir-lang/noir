#include "ecdsa.hpp"
#include <barretenberg/ecc/curves/secp256k1/secp256k1.hpp>

WASM_EXPORT void ecdsa__compute_public_key(uint8_t const* private_key, uint8_t* public_key_buf)
{
    auto priv_key = from_buffer<secp256k1::fr>(private_key);
    secp256k1::g1::affine_element pub_key = secp256k1::g1::one * priv_key;
    serialize::write(public_key_buf, pub_key);
}

WASM_EXPORT void ecdsa__construct_signature(uint8_t const* message,
                                            size_t msg_len,
                                            uint8_t const* private_key,
                                            uint8_t* output_sig_r,
                                            uint8_t* output_sig_s,
                                            uint8_t* output_sig_v)
{
    using serialize::write;
    auto priv_key = from_buffer<secp256k1::fr>(private_key);
    secp256k1::g1::affine_element pub_key = secp256k1::g1::one * priv_key;
    bb::crypto::ecdsa_key_pair<secp256k1::fr, secp256k1::g1> key_pair = { priv_key, pub_key };

    auto sig = bb::crypto::ecdsa_construct_signature<Sha256Hasher, secp256k1::fq, secp256k1::fr, secp256k1::g1>(
        std::string((char*)message, msg_len), key_pair);
    write(output_sig_r, sig.r);
    write(output_sig_s, sig.s);
    write(output_sig_v, sig.v);
}

WASM_EXPORT void ecdsa__recover_public_key_from_signature(uint8_t const* message,
                                                          size_t msg_len,
                                                          uint8_t const* sig_r,
                                                          uint8_t const* sig_s,
                                                          uint8_t* sig_v,
                                                          uint8_t* output_pub_key)
{
    std::array<uint8_t, 32> r, s;
    std::copy(sig_r, sig_r + 32, r.begin());
    std::copy(sig_s, sig_s + 32, s.begin());
    const uint8_t v = *sig_v;

    bb::crypto::ecdsa_signature sig = { r, s, v };
    auto recovered_pub_key =
        bb::crypto::ecdsa_recover_public_key<Sha256Hasher, secp256k1::fq, secp256k1::fr, secp256k1::g1>(
            std::string((char*)message, msg_len), sig);
    serialize::write(output_pub_key, recovered_pub_key);
}

WASM_EXPORT bool ecdsa__verify_signature(uint8_t const* message,
                                         size_t msg_len,
                                         uint8_t const* pub_key,
                                         uint8_t const* sig_r,
                                         uint8_t const* sig_s,
                                         uint8_t const* sig_v)
{
    auto pubk = from_buffer<secp256k1::g1::affine_element>(pub_key);
    std::array<uint8_t, 32> r, s;
    std::copy(sig_r, sig_r + 32, r.begin());
    std::copy(sig_s, sig_s + 32, s.begin());
    const uint8_t v = *sig_v;

    bb::crypto::ecdsa_signature sig = { r, s, v };
    return bb::crypto::ecdsa_verify_signature<Sha256Hasher, secp256k1::fq, secp256k1::fr, secp256k1::g1>(
        std::string((char*)message, msg_len), pubk, sig);
}
