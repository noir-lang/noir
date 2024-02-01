#pragma once

/**
 * @brief Provides interfaces for different PCS 'VerificationKey' classes.
 *
 */

#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"

#include <cstddef>
#include <memory>
#include <string_view>

namespace bb {

template <class Curve> class VerifierCommitmentKey;

/**
 * @brief Specialization for bn254
 *
 * @tparam curve::BN254
 */
template <> class VerifierCommitmentKey<curve::BN254> {
    using Curve = curve::BN254;
    using GroupElement = typename Curve::Element;
    using Commitment = typename Curve::AffineElement;

  public:
    VerifierCommitmentKey() = delete;

    /**
     * @brief Construct a new Kate Verification Key object from existing SRS
     *
     * @param num_points
     * @param srs verifier G2 point
     */
    VerifierCommitmentKey([[maybe_unused]] size_t num_points,
                          std::shared_ptr<bb::srs::factories::CrsFactory<Curve>> crs_factory)
        : srs(crs_factory->get_verifier_crs())
    {}

    /**
     * @brief verifies a pairing equation over 2 points using the verifier SRS
     *
     * @param p0 = P₀
     * @param p1 = P₁
     * @return e(P₀,[1]₁)e(P₁,[x]₂) ≡ [1]ₜ
     */
    bool pairing_check(const GroupElement& p0, const GroupElement& p1)
    {
        Commitment pairing_points[2]{ p0, p1 };
        // The final pairing check of step 12.
        Curve::TargetField result =
            bb::pairing::reduced_ate_pairing_batch_precomputed(pairing_points, srs->get_precomputed_g2_lines(), 2);

        return (result == Curve::TargetField::one());
    }

    std::shared_ptr<bb::srs::factories::VerifierCrs<Curve>> srs;
};

/**
 * @brief Specialization for Grumpkin
 *
 * @tparam curve::Grumpkin
 */
template <> class VerifierCommitmentKey<curve::Grumpkin> {
    using Curve = curve::Grumpkin;
    using GroupElement = typename Curve::Element;
    using Commitment = typename Curve::AffineElement;

  public:
    VerifierCommitmentKey() = delete;

    /**
     * @brief Construct a new IPA Verification Key object from existing SRS
     *
     *
     * @param num_points specifies the length of the SRS
     * @param path is the location to the SRS file
     */
    VerifierCommitmentKey(size_t num_points, std::shared_ptr<bb::srs::factories::CrsFactory<Curve>> crs_factory)
        : pippenger_runtime_state(num_points)
        , srs(crs_factory->get_verifier_crs(num_points))

    {}

    bb::scalar_multiplication::pippenger_runtime_state<Curve> pippenger_runtime_state;
    std::shared_ptr<bb::srs::factories::VerifierCrs<Curve>> srs;
};

} // namespace bb
