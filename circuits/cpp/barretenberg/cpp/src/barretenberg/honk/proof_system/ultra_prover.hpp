#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/honk/pcs/shplonk/shplonk.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include <array>
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/types/program_settings.hpp"
#include "barretenberg/honk/pcs/gemini/gemini.hpp"
#include "barretenberg/honk/pcs/shplonk/shplonk_single.hpp"
#include "barretenberg/honk/pcs/kzg/kzg.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include "barretenberg/honk/sumcheck/sumcheck_output.hpp"
#include <span>
#include <unordered_map>
#include <vector>
#include <algorithm>
#include <cstddef>
#include <memory>
#include <utility>
#include <string>
#include "barretenberg/honk/pcs/claim.hpp"
#include "barretenberg/honk/proof_system/prover_library.hpp"

namespace proof_system::honk {

// TODO(luke): The naming here is awkward. The Standard Honk prover is called "Prover" and aliased as StandardProver. To
// be consistent with that convention outside of the prover class itself, I've called this class UltraHonkProver and use
// the alias UltraProver externally. Resolve.
template <typename settings> class UltraHonkProver {

    using Fr = barretenberg::fr;
    using Polynomial = barretenberg::Polynomial<Fr>;
    using Commitment = barretenberg::g1::affine_element;
    using POLYNOMIAL = proof_system::honk::StandardArithmetization::POLYNOMIAL;

  public:
    UltraHonkProver(std::vector<barretenberg::polynomial>&& wire_polys,
                    std::shared_ptr<plonk::proving_key> input_key = nullptr);

    plonk::proof& export_proof();
    plonk::proof& construct_proof();

    ProverTranscript<Fr> transcript;

    std::vector<barretenberg::polynomial> wire_polynomials;

    std::shared_ptr<plonk::proving_key> key;

    work_queue<pcs::kzg::Params> queue;

  private:
    plonk::proof proof;
};

extern template class UltraHonkProver<plonk::ultra_settings>;

using UltraProver = UltraHonkProver<plonk::ultra_settings>;

} // namespace proof_system::honk
