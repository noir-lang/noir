#pragma once
#include "barretenberg/proof_system/flavor/flavor.hpp"
namespace proof_system::honk {
template <class Flavor> struct ProverFoldingResult {
  public:
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using FoldingParameters = typename Flavor::FoldingParameters;
    ProverPolynomials folded_prover_polynomials;
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/656): turn folding data into a struct
    std::vector<uint8_t> folding_data;
    FoldingParameters params;
};

template <class Flavor> struct VerifierFoldingResult {
    using FF = typename Flavor::FF;
    using VerificationKey = typename Flavor::VerificationKey;
    using FoldingParameters = typename Flavor::FoldingParameters;
    std::vector<FF> folded_public_inputs;
    VerificationKey folded_verification_key;
    FoldingParameters parameters;
};

/**
 * @brief The aggregated result from the prover and verifier after a round of folding, used to create a new Instance.
 *
 * @tparam Flavor
 */
template <class Flavor> struct FoldingResult {
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using VerificationKey = typename Flavor::VerificationKey;
    using FoldingParameters = typename Flavor::FoldingParameters;
    ProverPolynomials folded_prover_polynomials;
    std::vector<FF> folded_public_inputs;
    std::shared_ptr<VerificationKey> verification_key;
    FoldingParameters params;
};
} // namespace proof_system::honk