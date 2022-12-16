#pragma once
#include "account_tx.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <srs/reference_string/mem_reference_string.hpp>
#include <stdlib/types/types.hpp>

namespace rollup {
namespace proofs {
namespace account {

using namespace plonk::stdlib::types;

void init_proving_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory, bool mock);

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data);

void release_key();

void init_verification_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory);

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data);

void account_circuit(Composer& composer, account_tx const& tx);

UnrolledProver new_account_prover(account_tx const& tx, bool mock);

bool verify_proof(waffle::plonk_proof const& proof);

std::shared_ptr<waffle::proving_key> get_proving_key();

std::shared_ptr<waffle::verification_key> get_verification_key();

size_t get_number_of_gates();

} // namespace account
} // namespace proofs
} // namespace rollup
