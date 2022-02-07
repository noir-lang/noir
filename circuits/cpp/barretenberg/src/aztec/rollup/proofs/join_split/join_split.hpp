#pragma once
#include "join_split_tx.hpp"
#include <plonk/reference_string/mem_reference_string.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk::stdlib::merkle_tree;
using namespace plonk::stdlib::types::turbo;

void init_proving_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory, bool mock);

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data);

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data);

UnrolledProver new_join_split_prover(join_split_tx const& tx, bool mock);

bool verify_proof(waffle::plonk_proof const& proof);

std::shared_ptr<waffle::proving_key> get_proving_key();

std::shared_ptr<waffle::verification_key> get_verification_key();

} // namespace join_split
} // namespace proofs
} // namespace rollup
