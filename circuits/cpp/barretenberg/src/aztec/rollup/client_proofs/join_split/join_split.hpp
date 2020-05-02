#pragma once
#include "join_split_tx.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <plonk/reference_string/mem_reference_string.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void join_split_circuit(Composer& composer, tx_note const& note, crypto::schnorr::signature const& sig);

Prover new_join_split_prover(join_split_tx const& tx);

bool verify_proof(waffle::plonk_proof const& proof);

} // namespace create
} // namespace client_proofs
} // namespace rollup