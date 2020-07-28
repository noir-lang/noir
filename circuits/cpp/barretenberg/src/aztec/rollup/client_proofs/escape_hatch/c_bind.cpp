#include "c_bind.h"
#include "escape_hatch.hpp"
#include "sign_notes.hpp"
#include <common/streams.hpp>
#include <cstdint>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <sstream>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::client_proofs::escape_hatch;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void escape_hatch__init_proving_key(void* pippenger, const uint8_t* g2x)
{
    // We know that we don't actually need any CRS to create a proving key, so just feed in a nothing.
    // Hacky, but, right now it needs *something*.
    if (!pippenger) {
        auto crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
        init_proving_key(std::move(crs_factory));
    } else {
        auto crs_factory = std::make_unique<waffle::PippengerReferenceStringFactory>(
            reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger), g2x);
        init_proving_key(std::move(crs_factory));
    }
}

WASM_EXPORT void escape_hatch__init_proving_key_from_buffer(uint8_t const* pk_buf)
{
    std::shared_ptr<waffle::ProverReferenceString> crs;
    waffle::proving_key_data pk_data;
    read(pk_buf, pk_data);
    init_proving_key(crs, std::move(pk_data));
}

WASM_EXPORT uint32_t escape_hatch__get_new_proving_key_data(uint8_t** output)
{
    // Computing the size of the serialized key is non trivial. We know it's ~700mb.
    // Allocate a buffer large enough to hold it, and abort if we overflow.
    // This is to keep memory usage down.
    size_t total_buf_len = 700 * 1024 * 1024;
    auto raw_buf = (uint8_t*)malloc(total_buf_len);
    if (raw_buf == 0) {
        info("Failed to allocate: ", total_buf_len);
        std::abort();
    }
    auto raw_buf_end = raw_buf;
    write(raw_buf_end, *get_proving_key());
    *output = raw_buf;
    auto len = static_cast<uint32_t>(raw_buf_end - raw_buf);
    if (len > total_buf_len) {
        info("Buffer overflow serializing proving key.");
        std::abort();
    }
    return len;
}

WASM_EXPORT void escape_hatch__init_verification_key(void* pippenger, uint8_t const* g2x)
{
    auto crs_factory = std::make_unique<waffle::PippengerReferenceStringFactory>(
        reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger), g2x);
    init_verification_key(std::move(crs_factory));
}

WASM_EXPORT void escape_hatch__init_verification_key_from_buffer(uint8_t const* vk_buf, uint8_t const* g2x)
{
    auto crs = std::make_shared<waffle::VerifierMemReferenceString>(g2x);
    waffle::verification_key_data vk_data;
    read(vk_buf, vk_data);
    init_verification_key(crs, std::move(vk_data));
}

WASM_EXPORT uint32_t escape_hatch__get_new_verification_key_data(uint8_t** output)
{
    auto buffer = to_buffer(*get_verification_key());
    auto raw_buf = (uint8_t*)malloc(buffer.size());
    memcpy(raw_buf, (void*)buffer.data(), buffer.size());
    *output = raw_buf;
    return static_cast<uint32_t>(buffer.size());
}

WASM_EXPORT void escape_hatch__encrypt_note(uint8_t const* note_buffer, uint8_t* output)
{
    tx_note note = from_buffer<tx_note>(note_buffer);
    auto encrypted = encrypt_note(note);
    write(output, encrypted);
}

WASM_EXPORT bool escape_hatch__decrypt_note(uint8_t const* encrypted_note_buf,
                                            uint8_t const* private_key_buf,
                                            uint8_t const* viewing_key_buf,
                                            uint8_t* output)
{
    grumpkin::g1::affine_element encrypted_note;
    read(encrypted_note_buf, encrypted_note.x);
    read(encrypted_note_buf, encrypted_note.y);
    grumpkin::fr private_key;
    read(private_key_buf, private_key);
    fr viewing_key;
    read(viewing_key_buf, viewing_key);
    uint256_t result;
    bool success = decrypt_note(encrypted_note, private_key, viewing_key, result);
    write(output, result);
    return success;
}

WASM_EXPORT void escape_hatch__sign_2_notes(uint8_t const* note_buffer, uint8_t* pk_buffer, uint8_t* output)
{
    auto private_key = grumpkin::fr::serialize_from_buffer(pk_buffer);
    grumpkin::g1::affine_element public_key = grumpkin::g1::one * private_key;
    auto notes = from_buffer<std::array<tx_note, 2>>(note_buffer);
    auto signature = sign_notes(notes, { private_key, public_key });
    write(output, signature);
}

WASM_EXPORT void* escape_hatch__new_prover(uint8_t const* escape_hatch_buf)
{
    auto tx = from_buffer<escape_hatch_tx>(escape_hatch_buf);
    auto prover = new_escape_hatch_prover(tx);
    auto heapProver = new UnrolledProver(std::move(prover));
    return heapProver;
}

WASM_EXPORT uint32_t escape_hatch__create_escape_hatch_proof(uint8_t const* escape_hatch_buf, void** output)
{
    auto tx = from_buffer<escape_hatch_tx>(escape_hatch_buf);
    auto proof = create_escape_hatch_proof(tx);
    auto heapProof = aligned_alloc(64, proof.size());
    memcpy(heapProof, proof.data(), proof.size());
    *output = heapProof;
    return uint32_t(proof.size());
}

WASM_EXPORT void escape_hatch__delete_prover(void* prover)
{
    delete reinterpret_cast<Prover*>(prover);
}

WASM_EXPORT bool escape_hatch__verify_proof(uint8_t* proof, uint32_t length)
{
    waffle::plonk_proof pp = { std::vector<uint8_t>(proof, proof + length) };
    return verify_proof(pp);
}
}
