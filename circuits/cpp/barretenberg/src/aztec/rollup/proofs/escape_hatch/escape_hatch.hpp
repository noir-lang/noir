#pragma once
#include "escape_hatch_tx.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <plonk/reference_string/mem_reference_string.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace escape_hatch {

using namespace plonk::stdlib::types::turbo;

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data);

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data);

UnrolledProver new_escape_hatch_prover(escape_hatch_tx const& tx);

std::vector<uint8_t> create_escape_hatch_proof(escape_hatch_tx const& tx);

bool verify_proof(waffle::plonk_proof const& proof);

std::shared_ptr<waffle::proving_key> get_proving_key();

std::shared_ptr<waffle::verification_key> get_verification_key();

} // namespace escape_hatch
} // namespace proofs
} // namespace rollup
