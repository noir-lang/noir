#pragma once
#include "barretenberg/common/mem.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/bn254/g2.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <cstddef>

namespace barretenberg::pairing {
struct miller_lines;
} // namespace barretenberg::pairing

namespace barretenberg::srs::factories {

/**
 * A prover crs representation.
 */
template <typename Curve> class ProverCrs {
  public:
    virtual ~ProverCrs() = default;
    ;

    /**
     *  @brief Returns the monomial points in a form to be consumed by scalar_multiplication pippenger algorithm.
     */
    virtual typename Curve::AffineElement* get_monomial_points() = 0;
    virtual size_t get_monomial_size() const = 0;
};

template <typename Curve> class VerifierCrs {
  public:
    virtual ~VerifierCrs() = default;
};
template <> class VerifierCrs<curve::BN254> {
    using Curve = curve::BN254;

  public:
    virtual Curve::G2AffineElement get_g2x() const = 0;
    /**
     * @brief As the G_2 element of the CRS is fixed, we can precompute the operations performed on it during the
     * pairing algorithm to optimise pairing computations.
     */
    virtual barretenberg::pairing::miller_lines const* get_precomputed_g2_lines() const = 0;
    /**
     *  @brief Returns the first G_1 element from the CRS, used by the Shplonk verifier to compute the final
     * commtiment.
     */
    virtual Curve::AffineElement get_first_g1() const = 0;
};

template <> class VerifierCrs<curve::Grumpkin> {
    using Curve = curve::Grumpkin;

  public:
    /**
     * @brief Returns the G_1 elements in the CRS after the pippenger point table has been applied on them
     *
     */
    virtual Curve::AffineElement* get_monomial_points() const = 0;
    virtual size_t get_monomial_size() const = 0;
    /**
     * @brief Returns the first G_1 element from the CRS, used by the Shplonk verifier to compute the final
     * commtiment.
     */
    virtual Curve::AffineElement get_first_g1() const = 0;
};

/**
 * A factory class to return the prover crs and verifier crs on request.
 * You can construct an empty placeholder factory, because composers need to be given a factory at construction time.
 */
template <typename Curve> class CrsFactory {
  public:
    CrsFactory() = default;
    CrsFactory(CrsFactory&& other) = default;
    virtual ~CrsFactory() = default;
    virtual std::shared_ptr<barretenberg::srs::factories::ProverCrs<Curve>> get_prover_crs(size_t) { return nullptr; }
    virtual std::shared_ptr<barretenberg::srs::factories::VerifierCrs<Curve>> get_verifier_crs(
        [[maybe_unused]] size_t degree = 0)
    {
        return nullptr;
    }
};

} // namespace barretenberg::srs::factories