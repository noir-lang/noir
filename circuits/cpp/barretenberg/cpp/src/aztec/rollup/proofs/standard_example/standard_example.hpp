#pragma once
#include <srs/reference_string/mem_reference_string.hpp>
#include <plonk/composer/standard_composer.hpp>

namespace rollup {
namespace proofs {
namespace standard_example {

using Composer = waffle::StandardComposer;
using Prover = waffle::Prover;

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void build_circuit(Composer& composer);

Prover new_prover();

bool verify_proof(waffle::plonk_proof const& proof);

} // namespace standard_example
} // namespace proofs
} // namespace rollup