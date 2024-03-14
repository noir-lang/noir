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

#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
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

    OinkProver(const std::shared_ptr<ProverInstance_<Flavor>>& inst,
               const std::shared_ptr<typename Flavor::CommitmentKey>& commitment_key,
               const std::shared_ptr<typename Flavor::Transcript>& transcript,
               std::string domain_separator = "")
        : instance(inst)
        , transcript(transcript)
        , commitment_key(commitment_key)
        , domain_separator(std::move(domain_separator))
    {
        instance->initialize_prover_polynomials();
    }

    void execute_preamble_round();
    void execute_wire_commitments_round();
    void execute_sorted_list_accumulator_round();
    void execute_log_derivative_inverse_round();
    void execute_grand_product_computation_round();
};
} // namespace bb