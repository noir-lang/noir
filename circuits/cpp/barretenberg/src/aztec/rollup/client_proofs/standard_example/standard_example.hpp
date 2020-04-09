#pragma once
#include <plonk/reference_string/mem_reference_string.hpp>
#include <stdlib/types/standard.hpp>

namespace rollup {
namespace client_proofs {
namespace standard_example {

using namespace plonk::stdlib::types::standard;

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void build_circuit(Composer& composer);

Prover new_prover();

bool verify_proof(waffle::plonk_proof const& proof);

} // namespace create
} // namespace client_proofs
} // namespace rollup