#pragma once
#include "account_tx.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <plonk/reference_string/mem_reference_string.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace account {

using namespace plonk::stdlib::types::turbo;

void init_proving_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory);

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data);

void init_verification_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory);

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data);

void account_circuit(Composer& composer, account_tx const& tx);

UnrolledProver new_account_prover(account_tx const& tx);

bool verify_proof(waffle::plonk_proof const& proof);

std::shared_ptr<waffle::proving_key> get_proving_key();

std::shared_ptr<waffle::verification_key> get_verification_key();

} // namespace account
} // namespace proofs
} // namespace rollup
