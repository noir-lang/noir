#include "c_bind.h"
#include "create.hpp"
#include <common/streams.hpp>
#include <cstdint>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <sstream>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

#define WASM_EXPORT __attribute__((visibility("default")))

rollup::tx::tx_note create_tx_note(uint8_t const* owner_buf, uint32_t value, uint8_t const* viewing_key_buf)
{
    grumpkin::g1::affine_element owner =
        grumpkin::g1::affine_element::serialize_from_buffer(const_cast<uint8_t*>(owner_buf));
    fr viewing_key = fr::serialize_from_buffer(const_cast<uint8_t*>(viewing_key_buf));
    return { owner, value, viewing_key };
}

extern "C" {

WASM_EXPORT void create_note__init_proving_key()
{
    // We know that we don't actually need any CRS to create a proving key, so just feed in a nothing.
    // Hacky, but, right now it needs *something*.
    auto crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
    rollup::client_proofs::create_note::init_proving_key(std::move(crs_factory));
}

WASM_EXPORT void create_note__init_verification_key(void* pippenger, uint8_t const* g2x)
{
    auto crs_factory = std::make_unique<waffle::PippengerReferenceStringFactory>(
        reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger), g2x);
    rollup::client_proofs::create_note::init_verification_key(std::move(crs_factory));
}

WASM_EXPORT void create_note__encrypt_note(uint8_t const* owner_buf, uint32_t value, uint8_t const* viewing_key_buf, uint8_t* output)
{
    auto note = create_tx_note(owner_buf, value, viewing_key_buf);
    auto encrypted = rollup::tx::encrypt_note(note);
    grumpkin::g1::affine_element::serialize_to_buffer(encrypted, output);
}

WASM_EXPORT void* create_note__new_prover(uint8_t const* owner_buf,
                                         uint32_t value,
                                         uint8_t const* viewing_key_buf,
                                         uint8_t const* sig_s,
                                         uint8_t const* sig_e)
{
    auto note = create_tx_note(owner_buf, value, viewing_key_buf);

    std::array<uint8_t, 32> s, e;
    std::copy(sig_s, sig_s + 32, s.begin());
    std::copy(sig_e, sig_e + 32, e.begin());
    crypto::schnorr::signature sig = { s, e };

    auto prover = rollup::client_proofs::create_note::new_create_note_prover(note, sig);

    auto heapProver = new Prover(std::move(prover));
    return heapProver;
}

WASM_EXPORT void create_note__delete_prover(void* prover)
{
    delete reinterpret_cast<Prover*>(prover);
}

WASM_EXPORT bool create_note__verify_proof(uint8_t* proof, uint32_t length)
{
    waffle::plonk_proof pp = { std::vector<uint8_t>(proof, proof + length) };
    return rollup::client_proofs::create_note::verify_proof(pp);
}
}
