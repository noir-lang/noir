#pragma once
#include "crs_factory.hpp"
#include <cstddef>
#include <utility>

namespace bb::srs::factories {

/**
 * Create reference strings given pointers to in memory buffers.
 *
 * This class is currently only used with wasm and works exclusively with the Grumpkin CRS.
 */
class MemGrumpkinCrsFactory : public CrsFactory<curve::Grumpkin> {
  public:
    MemGrumpkinCrsFactory(std::vector<curve::Grumpkin::AffineElement> const& points);
    MemGrumpkinCrsFactory(MemGrumpkinCrsFactory&& other) = default;

    std::shared_ptr<bb::srs::factories::ProverCrs<curve::Grumpkin>> get_prover_crs(size_t degree) override;

    std::shared_ptr<bb::srs::factories::VerifierCrs<curve::Grumpkin>> get_verifier_crs(size_t degree = 0) override;

  private:
    std::shared_ptr<bb::srs::factories::ProverCrs<curve::Grumpkin>> prover_crs_;
    std::shared_ptr<bb::srs::factories::VerifierCrs<curve::Grumpkin>> verifier_crs_;
};

} // namespace bb::srs::factories
