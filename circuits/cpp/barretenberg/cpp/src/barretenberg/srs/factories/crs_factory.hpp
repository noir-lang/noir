#pragma once
#include "barretenberg/common/mem.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/bn254/g2.hpp"
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
     * Returns the monomial points in a form to be consumed by scalar_multiplication pippenger algorithm.
     */
    virtual typename Curve::AffineElement* get_monomial_points() = 0;
    virtual size_t get_monomial_size() const = 0;
};

class VerifierCrs {
  public:
    virtual ~VerifierCrs() = default;
    ;

    virtual barretenberg::g2::affine_element get_g2x() const = 0;

    virtual barretenberg::pairing::miller_lines const* get_precomputed_g2_lines() const = 0;
};

/**
 * A factory class to return the prover crs and verifier crs on request.
 * You can construct an empty placeholder factory, because composers need to be given a factory at construction time.
 */
class CrsFactory {
  public:
    CrsFactory() = default;
    CrsFactory(CrsFactory&& other) = default;
    virtual ~CrsFactory() = default;
    virtual std::shared_ptr<barretenberg::srs::factories::ProverCrs<curve::BN254>> get_prover_crs(size_t)
    {
        return nullptr;
    }
    virtual std::shared_ptr<barretenberg::srs::factories::VerifierCrs> get_verifier_crs() { return nullptr; }
};

} // namespace barretenberg::srs::factories
