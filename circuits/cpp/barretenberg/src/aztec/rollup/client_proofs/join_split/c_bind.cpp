#include "c_bind.h"
#include "join_split.hpp"
#include "sign_notes.hpp"
#include <common/streams.hpp>
#include <cstdint>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <sstream>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::client_proofs::join_split;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void join_split__init_proving_key()
{
    // We know that we don't actually need any CRS to create a proving key, so just feed in a nothing.
    // Hacky, but, right now it needs *something*.
    auto crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
    init_proving_key(std::move(crs_factory));
}

WASM_EXPORT void join_split__init_verification_key(void* pippenger, uint8_t const* g2x)
{
    auto crs_factory = std::make_unique<waffle::PippengerReferenceStringFactory>(
        reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger), g2x);
    init_verification_key(std::move(crs_factory));
}

WASM_EXPORT void join_split__sign_4_notes(uint8_t* note_buffer, uint8_t* pk_buffer, uint8_t* output)
{
    auto private_key = grumpkin::fr::serialize_from_buffer(pk_buffer);
    grumpkin::g1::affine_element public_key = grumpkin::g1::one * private_key;
    std::array<tx_note, 4> notes;
    read(note_buffer, notes);
    auto signature = sign_notes(notes, { private_key, public_key });
    info(signature);
    write(output, signature);
}

WASM_EXPORT void join_split__encrypt_note(uint8_t* note_buffer, uint8_t* output)
{
    tx_note note;
    read(note_buffer, note);
    auto encrypted = encrypt_note(note);
    grumpkin::g1::affine_element::serialize_to_buffer(encrypted, output);
}

WASM_EXPORT void* join_split__new_prover(uint8_t* join_split_buf)
{
    auto tx = join_split_tx::from_buffer(join_split_buf);
    info(tx);
    return 0;

    // auto prover = new_join_split_prover(note, sig);

    // auto heapProver = new Prover(std::move(prover));
    // return heapProver;
}

WASM_EXPORT void join_split__delete_prover(void* prover)
{
    delete reinterpret_cast<Prover*>(prover);
}

WASM_EXPORT bool join_split__verify_proof(uint8_t* proof, uint32_t length)
{
    waffle::plonk_proof pp = { std::vector<uint8_t>(proof, proof + length) };
    return verify_proof(pp);
}
}
