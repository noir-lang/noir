#pragma once
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/bn254/g2.hpp"
#include "crs_factory.hpp"
#include <cstddef>
#include <utility>

namespace barretenberg::srs::factories {

/**
 * Create reference strings given pointers to in memory buffers.
 *
 * This class is currently only used with wasm and works exclusively with the BN254 CRS.
 */
class MemCrsFactory : public CrsFactory<curve::BN254> {
  public:
    MemCrsFactory(std::vector<g1::affine_element> const& points, g2::affine_element const g2_point);
    MemCrsFactory(MemCrsFactory&& other) = default;

    std::shared_ptr<barretenberg::srs::factories::ProverCrs<curve::BN254>> get_prover_crs(size_t degree) override;

    std::shared_ptr<barretenberg::srs::factories::VerifierCrs<curve::BN254>> get_verifier_crs(
        size_t degree = 0) override;

  private:
    std::shared_ptr<barretenberg::srs::factories::ProverCrs<curve::BN254>> prover_crs_;
    std::shared_ptr<barretenberg::srs::factories::VerifierCrs<curve::BN254>> verifier_crs_;
};

} // namespace barretenberg::srs::factories
