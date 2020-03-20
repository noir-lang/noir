#include <cstdint>
#include <sstream>
#include <plonk/reference_string/mem_reference_string.hpp>
#include "create.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

rollup::tx::tx_note create_note(uint8_t const* owner_buf, uint32_t value, uint8_t const* viewing_key_buf)
{
    grumpkin::g1::affine_element owner;
    grumpkin::g1::affine_element::serialize_from_buffer(const_cast<uint8_t*>(owner_buf));
    barretenberg::fr viewing_key;
    barretenberg::fr::serialize_from_buffer(const_cast<uint8_t*>(viewing_key_buf));
    rollup::tx::tx_note note = { owner, value, viewing_key };
    return note;
}

extern "C" {

WASM_EXPORT void create_note_proof(uint8_t const* owner_buf,
                                   uint32_t value,
                                   uint8_t const* viewing_key_buf,
                                   uint8_t const* sig_s,
                                   uint8_t const* sig_e,
                                   uint8_t const* monomials_buf,
                                   uint32_t monomials_buf_size,
                                   uint8_t* proof_data_buf)
{
    auto note = create_note(owner_buf, value, viewing_key_buf);

    std::array<uint8_t, 32> s, e;
    std::copy(sig_s, sig_s + 32, s.begin());
    std::copy(sig_e, sig_e + 32, e.begin());
    crypto::schnorr::signature sig = { s, e };

    auto crs_factory =
        std::make_unique<waffle::MemReferenceStringFactory>((char*)monomials_buf, monomials_buf_size);

    auto proof_data = rollup::client_proofs::create::create_note_proof(note, sig, std::move(crs_factory));
    std::copy(proof_data.begin(), proof_data.end(), proof_data_buf);
}
}
