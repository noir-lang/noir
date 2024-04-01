#pragma once
// clang-format off
/*                                            )\   /|
*                                          .-/'-|_/ |
*                       __            __,-' (   / \/          
*                   .-'"  "'-..__,-'""          -o.`-._   
*                  /                                   '/
*          *--._ ./                                 _.-- 
*                |                              _.-' 
*                :                           .-/   
*                 \                       )_ /
*                  \                _)   / \(
*                    `.   /-.___.---'(  /   \\
*                     (  /   \\       \(     L\
*                      \(     L\       \\
*                       \\              \\
*                        L\              L\
*/
// clang-format on
#include <utility>

#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {
template <IsUltraFlavor Flavor> struct OinkProverOutput {
    typename Flavor::ProvingKey proving_key;
    bb::RelationParameters<typename Flavor::FF> relation_parameters;
    typename Flavor::RelationSeparator alphas;
};

/**
 * @brief Class for all the oink rounds, which are shared between the folding prover and ultra prover.
 * @details This class contains execute_preamble_round(), execute_wire_commitments_round(),
 * execute_sorted_list_accumulator_round(), execute_log_derivative_inverse_round(), and
 * execute_grand_product_computation_round().
 *
 * @tparam Flavor
 */
template <IsUltraFlavor Flavor> class OinkProver {
    using CommitmentKey = typename Flavor::CommitmentKey;
    using ProvingKey = typename Flavor::ProvingKey;
    using Transcript = typename Flavor::Transcript;
    using FF = typename Flavor::FF;

  public:
    ProvingKey proving_key;
    std::shared_ptr<Transcript> transcript;
    std::shared_ptr<CommitmentKey> commitment_key;
    std::string domain_separator;
    typename Flavor::WitnessCommitments witness_commitments;
    typename Flavor::CommitmentLabels commitment_labels;
    using RelationSeparator = typename Flavor::RelationSeparator;

    bb::RelationParameters<typename Flavor::FF> relation_parameters;

    OinkProver(ProvingKey& proving_key,
               const std::shared_ptr<typename Flavor::Transcript>& transcript,
               std::string domain_separator = "")
        : proving_key(std::move(proving_key))
        , transcript(transcript)
        , commitment_key(this->proving_key.commitment_key)
        , domain_separator(std::move(domain_separator))
    {}

    OinkProverOutput<Flavor> prove();
    void execute_preamble_round();
    void execute_wire_commitments_round();
    void execute_sorted_list_accumulator_round();
    void execute_log_derivative_inverse_round();
    void execute_grand_product_computation_round();
    RelationSeparator generate_alphas_round();
};
} // namespace bb