#pragma once
#include "barretenberg/join_split_example/types.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include "join_split_tx.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

void init_proving_key(std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory,
                      bool mock);

void release_proving_key();

void init_verification_key(std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory);

Prover new_join_split_prover(join_split_tx const& tx, bool mock);

bool verify_proof(plonk::proof const& proof);

std::shared_ptr<plonk::proving_key> get_proving_key();

std::shared_ptr<plonk::verification_key> get_verification_key();

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
