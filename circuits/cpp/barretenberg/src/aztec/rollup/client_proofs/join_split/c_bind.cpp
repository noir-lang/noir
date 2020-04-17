#include "c_bind.h"
#include "join_split.hpp"
#include <common/streams.hpp>
#include <cstdint>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <sstream>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void join_split__init_proving_key()
{
    // We know that we don't actually need any CRS to create a proving key, so just feed in a nothing.
    // Hacky, but, right now it needs *something*.
    auto crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
    rollup::client_proofs::join_split::init_proving_key(std::move(crs_factory));
}

WASM_EXPORT void join_split__init_verification_key(void* pippenger, uint8_t const* g2x)
{
    auto crs_factory = std::make_unique<waffle::PippengerReferenceStringFactory>(
        reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger), g2x);
    rollup::client_proofs::join_split::init_verification_key(std::move(crs_factory));
}

WASM_EXPORT void join_split__encrypt_note(uint8_t* note_buffer, uint8_t* output)
{
    auto note = rollup::client_proofs::join_split::deserialize_tx_note(note_buffer);
    auto encrypted = rollup::client_proofs::join_split::encrypt_note(note);
    grumpkin::g1::affine_element::serialize_to_buffer(encrypted, output);
}

WASM_EXPORT void* join_split__new_prover(uint8_t* join_split_buf, uint32_t buf_length)
{
    info("ENTERING JSNP");
    auto tx =
        rollup::client_proofs::join_split::join_split_tx::from_buffer({ join_split_buf, join_split_buf + buf_length });

    info(tx);

    return 0;

    // auto prover = rollup::client_proofs::join_split::new_join_split_prover(note, sig);

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
    return rollup::client_proofs::join_split::verify_proof(pp);
}
}
