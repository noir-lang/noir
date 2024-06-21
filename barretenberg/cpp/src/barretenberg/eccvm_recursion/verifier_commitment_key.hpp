#pragma once
#include "barretenberg/commitment_schemes/verification_key.hpp"
namespace bb {

/**
 * @brief Representation of the Grumpkin Verifier Commitment Key inside a bn254 circuit
 *
 * @tparam Builder
 */
template <typename Curve> class VerifierCommitmentKey {
    using Builder = Curve::Builder;
    using Commitment = Curve::AffineElement;
    using NativeEmbeddedCurve = typename Builder::EmbeddedCurve;

  public:
    /**
     * @brief Construct a new Verifier Commitment Key object from its native counterpart. instantiated on Grumpkin.
     * This will be part of the ECCVMRecursiveFlavor once implemented. The Grumpkin SRS points are represented after
     * applying the pippenger point table so the values at odd indices contain the point {srs[i-1].x * beta,
     * srs[i-1].y}, where beta is the endomorphism. We retrieve only the original SRS for IPA verification.
     *
     * @details The Grumpkin SRS points will be initialised as constants in the circuit but might be subsequently
     * turned into constant witnesses to make operations in the circuit more efficient.
     */
    VerifierCommitmentKey([[maybe_unused]] Builder* builder,
                          size_t num_points,
                          std::shared_ptr<VerifierCommitmentKey<NativeEmbeddedCurve>>& native_pcs_verification_key)
        : g1_identity(Commitment(native_pcs_verification_key->get_g1_identity()))
    {

        auto* native_points = native_pcs_verification_key->get_monomial_points();
        for (size_t i = 0; i < num_points * 2; i += 2) {
            monomial_points.emplace_back(Commitment(native_points[i]));
        }
    }

    Commitment get_g1_identity() { return g1_identity; }
    std::vector<Commitment> get_monomial_points() { return monomial_points; }

  private:
    Commitment g1_identity;
    std::vector<Commitment> monomial_points;
};
} // namespace bb