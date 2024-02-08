#pragma once

#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

namespace bb {

/**
 * @brief The IVC interface to be used by the aztec client for private function execution
 * @details Combines Protogalaxy with Goblin to accumulate one circuit instance at a time with efficient EC group
 * operations
 *
 */
class ClientIVC {

  public:
    using Flavor = GoblinUltraFlavor;
    using FF = Flavor::FF;
    using FoldProof = std::vector<FF>;
    using Accumulator = std::shared_ptr<ProverInstance_<Flavor>>;
    using ClientCircuit = GoblinUltraCircuitBuilder; // can only be GoblinUltra

    // A full proof for the IVC scheme
    struct Proof {
        FoldProof fold_proof; // final fold proof
        HonkProof decider_proof;
        Goblin::Proof goblin_proof;
    };

  private:
    using FoldingOutput = FoldingResult<Flavor>;
    using Instance = ProverInstance_<GoblinUltraFlavor>;
    using Composer = GoblinUltraComposer;

  public:
    Goblin goblin;
    FoldingOutput fold_output;

    ClientIVC();

    void initialize(ClientCircuit& circuit);

    FoldProof accumulate(ClientCircuit& circuit);

    Proof prove();

    bool verify(Proof& proof);

    HonkProof decider_prove() const;
};
} // namespace bb