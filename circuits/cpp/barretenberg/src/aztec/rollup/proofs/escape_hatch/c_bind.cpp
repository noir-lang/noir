#include "c_bind.h"
#include "escape_hatch.hpp"
#include <common/streams.hpp>
#include <cstdint>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <sstream>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs::escape_hatch;
using namespace rollup::proofs::join_split;

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

WASM_EXPORT void escape_hatch__init_verification_key(void* pippenger, uint8_t const* g2x)
{
    auto crs_factory = std::make_unique<waffle::PippengerReferenceStringFactory>(
        reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger), g2x);
    init_verification_key(std::move(crs_factory));
}

WASM_EXPORT void* escape_hatch__new_prover(uint8_t const* escape_hatch_buf)
{
    auto tx = from_buffer<escape_hatch_tx>(escape_hatch_buf);
    auto prover = new_escape_hatch_prover(tx);
    auto heapProver = new Prover(std::move(prover));
    return heapProver;
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
