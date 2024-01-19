#pragma once
#include "barretenberg/join_split_example/types.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include "join_split_tx.hpp"

namespace bb::join_split_example::proofs::join_split {

void init_proving_key(bool mock);

void release_proving_key();

void init_verification_key();

Prover new_join_split_prover(join_split_tx const& tx, bool mock);

bool verify_proof(plonk::proof const& proof);

std::shared_ptr<plonk::proving_key> get_proving_key();

std::shared_ptr<plonk::verification_key> get_verification_key();

} // namespace bb::join_split_example::proofs::join_split
