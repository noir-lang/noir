#pragma once

/**
 * @brief Provides interfaces for different 'CommitmentKey' classes.
 *
 * TODO(#218)(Mara): This class should handle any modification to the SRS (e.g compute pippenger point table) to
 * simplify the codebase.
 */

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

namespace proof_system::honk::pcs {

namespace kzg {

struct Params {
    using Curve = curve::BN254;
    using Fr = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;
    using GroupElement = typename Curve::Element;

    using Polynomial = barretenberg::Polynomial<Fr>;

    class CommitmentKey;
    class VerificationKey;
    /**
     * @brief CommitmentKey object over a pairing group 𝔾₁, using a structured reference string (SRS).
     * The SRS is given as a list of 𝔾₁ points { [xʲ]₁ }ⱼ where 'x' is unknown. The SRS stored in the commitment key is
     * after applying the pippenger_point_table thus being double the size of what is loaded from path.
     *
     *
     */
    class CommitmentKey {

      public:
        CommitmentKey() = delete;

        /**
         * @brief Construct a new Kate Commitment Key object from existing SRS
         *
         * @param n
         * @param path
         *
         */
        CommitmentKey(const size_t num_points,
                      std::shared_ptr<barretenberg::srs::factories::CrsFactory<Curve>> crs_factory)
            : pippenger_runtime_state(num_points)
            , srs(crs_factory->get_prover_crs(num_points))
        {}

        // Note: This constructor is used only by Plonk; For Honk the CommitmentKey is solely responsible for extracting
        // the srs.
        CommitmentKey(const size_t num_points,
                      std::shared_ptr<barretenberg::srs::factories::ProverCrs<Curve>> prover_crs)
            : pippenger_runtime_state(num_points)
            , srs(prover_crs)
        {}

        /**
         * @brief Uses the ProverSRS to create a commitment to p(X)
         *
         * @param polynomial a univariate polynomial p(X) = ∑ᵢ aᵢ⋅Xⁱ ()
         * @return Commitment computed as C = [p(x)] = ∑ᵢ aᵢ⋅[xⁱ]₁ where x is the secret trapdoor
         */
        Commitment commit(std::span<const Fr> polynomial)
        {
            const size_t degree = polynomial.size();
            ASSERT(degree <= srs->get_monomial_size());
            return barretenberg::scalar_multiplication::pippenger_unsafe<Curve>(
                const_cast<Fr*>(polynomial.data()), srs->get_monomial_points(), degree, pippenger_runtime_state);
        };

        barretenberg::scalar_multiplication::pippenger_runtime_state<Curve> pippenger_runtime_state;
        std::shared_ptr<barretenberg::srs::factories::ProverCrs<Curve>> srs;
    };

    class VerificationKey {

      public:
        VerificationKey() = delete;

        /**
         * @brief Construct a new Kate Verification Key object from existing SRS
         *
         * @param num_points
         * @paramsrs verifier G2 point
         */
        VerificationKey([[maybe_unused]] size_t num_points,
                        std::shared_ptr<barretenberg::srs::factories::CrsFactory<Curve>> crs_factory)
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
            Curve::TargetField result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
                pairing_points, srs->get_precomputed_g2_lines(), 2);

            return (result == Curve::TargetField::one());
        }

        std::shared_ptr<barretenberg::srs::factories::VerifierCrs<Curve>> srs;
    };
};

} // namespace kzg

namespace fake {

// Define a common trapdoor for both keys
namespace {
template <typename G> constexpr typename G::Fr trapdoor(5);
}

template <typename G> struct Params {
    using Fr = typename G::Fr;
    using Commitment = typename G::affine_element;
    using GroupElement = typename G::element;

    using Polynomial = barretenberg::Polynomial<Fr>;

    template <G> class CommitmentKey;
    template <G> class VerificationKey;

    /**
     * @brief Simulates a KZG CommitmentKey, but where we know the secret trapdoor
     * which allows us to commit to polynomials using a single group multiplication.
     *
     * @tparam G the commitment group
     */
    template <G> class CommitmentKey {

      public:
        /**
         * @brief efficiently create a KZG commitment to p(X) using the trapdoor 'secret'
         * Uses only 1 group scalar multiplication, and 1 polynomial evaluation
         *
         *
         * @param polynomial a univariate polynomial p(X)
         * @return Commitment computed as C = p(secret)•[1]_1 .
         */
        Commitment commit(std::span<const Fr> polynomial)
        {
            const Fr eval_secret = barretenberg::polynomial_arithmetic::evaluate(polynomial, trapdoor<G>);
            return Commitment::one() * eval_secret;
        };
    };

    template <G> class VerificationKey {

      public:
        /**
         * @brief verifies a pairing equation over 2 points using the trapdoor
         *
         * @param p0 = P₀
         * @param p1 = P₁
         * @return P₀ - x⋅P₁ ≡ [1]
         */
        bool pairing_check(const Commitment& p0, const Commitment& p1)
        {
            Commitment result = p0 + p1 * trapdoor<G>;
            return result.is_point_at_infinity();
        }
    };
};
} // namespace fake

namespace ipa {

struct Params {
    using Curve = curve::Grumpkin;
    using Fr = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;
    using GroupElement = typename Curve::Element;

    using Polynomial = barretenberg::Polynomial<Fr>;

    class CommitmentKey;
    class VerificationKey;

    class CommitmentKey {

      public:
        CommitmentKey() = delete;

        /**
         * @brief Construct a new IPA Commitment Key object from existing SRS..
         *
         * @param num_points
         * @param path
         *
         */
        CommitmentKey(const size_t num_points,
                      std::shared_ptr<barretenberg::srs::factories::CrsFactory<Curve>> crs_factory)
            : pippenger_runtime_state(num_points)
            , srs(crs_factory->get_prover_crs(num_points))
        {}

        /**
         * @brief Uses the ProverSRS to create an unblinded commitment to p(X)
         *
         * @param polynomial a univariate polynomial p(X) = ∑ᵢ aᵢ⋅Xⁱ ()
         * @return Commitment computed as C = [p(x)] = ∑ᵢ aᵢ⋅Gᵢ where Gᵢ is the i-th element of the SRS
         */
        Commitment commit(std::span<const Fr> polynomial)
        {
            const size_t degree = polynomial.size();
            ASSERT(degree <= srs->get_monomial_size());
            return barretenberg::scalar_multiplication::pippenger_unsafe<Curve>(
                const_cast<Fr*>(polynomial.data()), srs->get_monomial_points(), degree, pippenger_runtime_state);
        };

        barretenberg::scalar_multiplication::pippenger_runtime_state<Curve> pippenger_runtime_state;
        std::shared_ptr<barretenberg::srs::factories::ProverCrs<Curve>> srs;
    };

    class VerificationKey {
      public:
        VerificationKey() = delete;

        /**
         * @brief Construct a new IPA Verification Key object from existing SRS
         *
         *
         * @param num_points specifies the length of the SRS
         * @param path is the location to the SRS file
         */
        VerificationKey(size_t num_points, std::shared_ptr<barretenberg::srs::factories::CrsFactory<Curve>> crs_factory)
            : pippenger_runtime_state(num_points)
            , srs(crs_factory->get_verifier_crs(num_points))

        {}

        barretenberg::scalar_multiplication::pippenger_runtime_state<Curve> pippenger_runtime_state;
        std::shared_ptr<barretenberg::srs::factories::VerifierCrs<Curve>> srs;
    };
};

} // namespace ipa

} // namespace proof_system::honk::pcs
