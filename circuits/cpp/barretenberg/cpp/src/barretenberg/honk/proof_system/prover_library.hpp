#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/types/program_settings.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

namespace proof_system::honk::prover_library {

template <typename Flavor>
typename Flavor::Polynomial compute_sorted_list_accumulator(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                                            typename Flavor::FF eta);

template <typename Flavor>
void add_plookup_memory_records_to_wire_4(std::shared_ptr<typename Flavor::ProvingKey>& key, typename Flavor::FF eta);

} // namespace proof_system::honk::prover_library
