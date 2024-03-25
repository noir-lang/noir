#pragma once

#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"

namespace bb {

template <IsUltraFlavor Flavor> struct OinkOutput {
    bb::RelationParameters<typename Flavor::FF> relation_parameters;
    typename Flavor::WitnessCommitments commitments;
    std::vector<typename Flavor::FF> public_inputs;
};

/**
 * @brief Verifier class for all the presumcheck rounds, which are shared between the folding verifier and ultra
 * verifier.
 * @details This class contains execute_preamble_round(), execute_wire_commitments_round(),
 * execute_sorted_list_accumulator_round(), execute_log_derivative_inverse_round(), and
 * execute_grand_product_computation_round().
 *
 * @tparam Flavor
 */
template <IsUltraFlavor Flavor> class OinkVerifier {
    using VerificationKey = typename Flavor::VerificationKey;
    using WitnessCommitments = typename Flavor::WitnessCommitments;
    using Transcript = typename Flavor::Transcript;
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;

  public:
    std::shared_ptr<Transcript> transcript;
    std::shared_ptr<VerificationKey> key;
    std::string domain_separator;
    typename Flavor::CommitmentLabels comm_labels;
    bb::RelationParameters<FF> relation_parameters;
    WitnessCommitments witness_comms;
    std::vector<FF> public_inputs;

    OinkVerifier(const std::shared_ptr<VerificationKey>& verifier_key,
                 const std::shared_ptr<Transcript>& transcript,
                 std::string domain_separator = "")
        : transcript(transcript)
        , key(verifier_key)
        , domain_separator(std::move(domain_separator))
    {}

    OinkOutput<Flavor> verify();

    void execute_preamble_round();

    void execute_wire_commitments_round();

    void execute_sorted_list_accumulator_round();

    void execute_log_derivative_inverse_round();

    void execute_grand_product_computation_round();
};
} // namespace bb