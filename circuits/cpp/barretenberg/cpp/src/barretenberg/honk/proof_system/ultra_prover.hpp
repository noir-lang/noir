#pragma once
#include "barretenberg/honk/proof_system/work_queue.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"

namespace proof_system::honk {

// We won't compile this class with honk::flavor::Standard, but we will like want to compile it (at least for testing)
// with a flavor that uses the curve Grumpkin, or a flavor that does/does not have zk, etc.
template <typename T> concept UltraFlavor = IsAnyOf<T, honk::flavor::Ultra>;
template <UltraFlavor Flavor> class UltraProver_ {

    using FF = typename Flavor::FF;
    using PCSParams = typename Flavor::PCSParams;
    using ProvingKey = typename Flavor::ProvingKey;
    using Polynomial = typename Flavor::Polynomial;

  public:
    UltraProver_(std::shared_ptr<ProvingKey> input_key = nullptr);

    plonk::proof& export_proof();
    plonk::proof& construct_proof();

    ProverTranscript<FF> transcript;

    std::shared_ptr<ProvingKey> key;

    work_queue<pcs::kzg::Params> queue;

  private:
    plonk::proof proof;
};

extern template class UltraProver_<honk::flavor::Ultra>;

using UltraProver = UltraProver_<honk::flavor::Ultra>;

} // namespace proof_system::honk
