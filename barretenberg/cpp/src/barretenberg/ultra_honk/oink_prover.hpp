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

#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_keccak.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {
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
    using Instance = ProverInstance_<Flavor>;
    using Transcript = typename Flavor::Transcript;
    using FF = typename Flavor::FF;

  public:
    std::shared_ptr<Instance> instance;
    std::shared_ptr<Transcript> transcript;
    std::shared_ptr<CommitmentKey> commitment_key;
    std::string domain_separator;
    typename Flavor::WitnessCommitments witness_commitments;
    typename Flavor::CommitmentLabels commitment_labels;
    using RelationSeparator = typename Flavor::RelationSeparator;

    OinkProver(std::shared_ptr<Instance> instance,
               const std::shared_ptr<typename Flavor::Transcript>& transcript = std::make_shared<Transcript>(),
               std::string domain_separator = "")
        : instance(instance)
        , transcript(transcript)
        , commitment_key(this->instance->proving_key.commitment_key)
        , domain_separator(std::move(domain_separator))
    {}

    void prove();
    void execute_preamble_round();
    void execute_wire_commitments_round();
    void execute_sorted_list_accumulator_round();
    void execute_log_derivative_inverse_round();
    void execute_grand_product_computation_round();
    RelationSeparator generate_alphas_round();
};
} // namespace bb