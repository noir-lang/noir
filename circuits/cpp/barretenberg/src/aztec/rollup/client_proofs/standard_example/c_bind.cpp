#include "c_bind.h"
#include "standard_example.hpp"
#include <common/streams.hpp>
#include <cstdint>
#include <plonk/reference_string/point_table_reference_string.hpp>
#include <sstream>

using namespace barretenberg;
using namespace plonk::stdlib::types::standard;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void standard_example_init_keys(uint8_t* point_table, uint32_t num_points, uint8_t const* g2x)
{
    auto crs_factory = std::make_unique<waffle::PointTableReferenceStringFactory>(
        reinterpret_cast<g1::affine_element*>(point_table), num_points, (char*)g2x);
    rollup::client_proofs::standard_example::init_keys(std::move(crs_factory));
}

WASM_EXPORT void standard_example_init_proving_key(uint8_t* point_table, uint32_t num_points)
{
    auto crs_factory = std::make_unique<waffle::PointTableReferenceStringFactory>(
        reinterpret_cast<g1::affine_element*>(point_table), num_points, (char*)0);
    rollup::client_proofs::standard_example::init_proving_key(std::move(crs_factory));
}

WASM_EXPORT void* standard_example_new_prover()
{
    auto prover = rollup::client_proofs::standard_example::new_prover();

    auto heapProver = new Prover(std::move(prover));
    return heapProver;
}

WASM_EXPORT void standard_example_delete_prover(void* prover)
{
    delete reinterpret_cast<Prover*>(prover);
}

WASM_EXPORT bool standard_example_verify_proof(uint8_t* proof, uint32_t length)
{
    waffle::plonk_proof pp = { std::vector<uint8_t>(proof, proof + length) };
    return rollup::client_proofs::standard_example::verify_proof(pp);
}
}
