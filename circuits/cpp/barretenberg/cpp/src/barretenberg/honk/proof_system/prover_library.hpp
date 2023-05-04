#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/types/program_settings.hpp"

namespace proof_system::honk::prover_library {

template <typename Flavor>
typename Flavor::Polynomial compute_permutation_grand_product(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                                              typename Flavor::FF beta,
                                                              typename Flavor::FF gamma);

template <typename Flavor>
typename Flavor::Polynomial compute_lookup_grand_product(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                                         typename Flavor::Polynomial& sorted_list_accumulator,
                                                         typename Flavor::FF eta,
                                                         typename Flavor::FF beta,
                                                         typename Flavor::FF gamma);

template <typename Flavor>
typename Flavor::Polynomial compute_sorted_list_accumulator(
    std::shared_ptr<typename Flavor::ProvingKey>& key,
    std::vector<typename Flavor::Polynomial>& sorted_list_polynomials,
    typename Flavor::FF eta);

} // namespace proof_system::honk::prover_library
