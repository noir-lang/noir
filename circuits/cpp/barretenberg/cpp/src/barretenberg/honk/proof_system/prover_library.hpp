#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/types/program_settings.hpp"

namespace proof_system::honk::prover_library {

using Fr = barretenberg::fr;
using Polynomial = barretenberg::Polynomial<Fr>;

template <size_t program_width>
Polynomial compute_permutation_grand_product(std::shared_ptr<plonk::proving_key>& key,
                                             std::vector<Polynomial>& wire_polynomials,
                                             Fr beta,
                                             Fr gamma);

Polynomial compute_lookup_grand_product(std::shared_ptr<plonk::proving_key>& key,
                                        std::vector<Polynomial>& wire_polynomials,
                                        Polynomial& sorted_list_accumulator,
                                        Fr eta,
                                        Fr beta,
                                        Fr gamma);

Polynomial compute_sorted_list_accumulator(std::shared_ptr<plonk::proving_key>& key,
                                           std::vector<Polynomial>& sorted_list_polynomials,
                                           Fr eta);

} // namespace proof_system::honk::prover_library
