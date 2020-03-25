#include <cstdint>
#include <sstream>
#include <plonk/reference_string/mem_reference_string.hpp>
#include "create.hpp"
#include <common/streams.hpp>

#define WASM_EXPORT __attribute__((visibility("default")))

rollup::tx::tx_note create_tx_note(uint8_t const* owner_buf, uint32_t value, uint8_t const* viewing_key_buf)
{
    grumpkin::g1::affine_element owner = grumpkin::g1::affine_element::serialize_from_buffer(const_cast<uint8_t*>(owner_buf));
    barretenberg::fr viewing_key = barretenberg::fr::serialize_from_buffer(const_cast<uint8_t*>(viewing_key_buf));
    return { owner, value, viewing_key };
}

extern "C" {

#ifdef __wasm__
void logstr(char const* str);
#else
inline void logstr(char const* str) {
    std::cout << str << std::endl;
}
#endif

WASM_EXPORT void init_keys(uint8_t const* monomials_buf, uint32_t monomials_buf_size, uint8_t const* g2x) {
    auto crs_factory = std::make_unique<waffle::MemReferenceStringFactory>((char*)monomials_buf, monomials_buf_size, (char*)g2x);
    rollup::client_proofs::create::init_keys(std::move(crs_factory));
}

WASM_EXPORT void init_proving_key(uint8_t const* monomials_buf, uint32_t monomials_buf_size) {
    auto crs_factory = std::make_unique<waffle::MemReferenceStringFactory>((char*)monomials_buf, monomials_buf_size, (char*)0);
    rollup::client_proofs::create::init_proving_key(std::move(crs_factory));
}

WASM_EXPORT void encrypt_note(uint8_t const* owner_buf, uint32_t value, uint8_t const* viewing_key_buf, uint8_t* output) {
    auto note = create_tx_note(owner_buf, value, viewing_key_buf);
    auto encrypted = rollup::tx::encrypt_note(note);
    grumpkin::g1::affine_element::serialize_to_buffer(encrypted, output);
}

WASM_EXPORT void create_note_proof(uint8_t const* owner_buf,
                                   uint32_t value,
                                   uint8_t const* viewing_key_buf,
                                   uint8_t const* sig_s,
                                   uint8_t const* sig_e,
                                   uint8_t* proof_data_buf)
{
    auto note = create_tx_note(owner_buf, value, viewing_key_buf);

    std::array<uint8_t, 32> s, e;
    std::copy(sig_s, sig_s + 32, s.begin());
    std::copy(sig_e, sig_e + 32, e.begin());
    crypto::schnorr::signature sig = { s, e };

    auto proof_data = rollup::client_proofs::create::create_note_proof(note, sig);
    *(uint32_t*)proof_data_buf = static_cast<uint32_t>(proof_data.size());
    std::copy(proof_data.begin(), proof_data.end(), proof_data_buf + 4);
}

WASM_EXPORT bool verify_proof(uint8_t* proof, uint32_t length) {
    waffle::plonk_proof pp = { std::vector<uint8_t>(proof, proof+length) };
    return rollup::client_proofs::create::verify_proof(pp);
}

}
