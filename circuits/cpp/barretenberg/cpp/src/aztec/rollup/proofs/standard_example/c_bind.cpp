#include "c_bind.h"
#include "standard_example.hpp"
#include <common/streams.hpp>
#include <cstdint>
#include <srs/reference_string/pippenger_reference_string.hpp>
#include <sstream>

using namespace barretenberg;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void standard_example__init_proving_key()
{
    auto crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
    rollup::proofs::standard_example::init_proving_key(std::move(crs_factory));
}

WASM_EXPORT void standard_example__init_verification_key(void* pippenger_ptr, uint8_t const* g2x)
{
    auto crs_factory = std::make_unique<waffle::PippengerReferenceStringFactory>(
        reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger_ptr), g2x);
    rollup::proofs::standard_example::init_verification_key(std::move(crs_factory));
}

WASM_EXPORT void* standard_example__new_prover()
{
    auto prover = rollup::proofs::standard_example::new_prover();
    return new waffle::Prover(std::move(prover));
}

WASM_EXPORT void standard_example__delete_prover(void* prover)
{
    delete reinterpret_cast<waffle::Prover*>(prover);
}

WASM_EXPORT bool standard_example__verify_proof(uint8_t* proof, uint32_t length)
{
    waffle::plonk_proof pp = { std::vector<uint8_t>(proof, proof + length) };
    return rollup::proofs::standard_example::verify_proof(pp);
}
}
