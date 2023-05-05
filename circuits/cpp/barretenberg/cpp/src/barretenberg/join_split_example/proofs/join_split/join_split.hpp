#pragma once
#include "join_split_tx.hpp"
#include "barretenberg/srs/reference_string/mem_reference_string.hpp"
#include "barretenberg/join_split_example/types.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

void init_proving_key(std::shared_ptr<proof_system::ReferenceStringFactory> const& crs_factory, bool mock);

void init_proving_key(std::shared_ptr<proof_system::ProverReferenceString> const& crs,
                      plonk::proving_key_data&& pk_data);

void release_key();

void init_verification_key(std::unique_ptr<proof_system::ReferenceStringFactory>&& crs_factory);

void init_verification_key(std::shared_ptr<proof_system::VerifierMemReferenceString> const& crs,
                           plonk::verification_key_data&& vk_data);

Prover new_join_split_prover(join_split_tx const& tx, bool mock);

bool verify_proof(plonk::proof const& proof);

std::shared_ptr<plonk::proving_key> get_proving_key();

std::shared_ptr<plonk::verification_key> get_verification_key();

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
