#pragma once

#include "barretenberg/common/mem.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/bn254/g2.hpp"

#include <cstddef>

namespace barretenberg::pairing {
struct miller_lines;
} // namespace barretenberg::pairing

namespace proof_system {

class VerifierReferenceString {
  public:
    virtual ~VerifierReferenceString() = default;
    ;

    virtual barretenberg::g2::affine_element get_g2x() const = 0;

    virtual barretenberg::pairing::miller_lines const* get_precomputed_g2_lines() const = 0;
};

class ProverReferenceString {
  public:
    virtual ~ProverReferenceString() = default;
    ;

    virtual barretenberg::g1::affine_element* get_monomial_points() = 0;
    virtual size_t get_monomial_size() const = 0;
};

class ReferenceStringFactory {
  public:
    ReferenceStringFactory() = default;
    ReferenceStringFactory(ReferenceStringFactory&& other) = default;
    virtual ~ReferenceStringFactory() = default;
    virtual std::shared_ptr<ProverReferenceString> get_prover_crs(size_t) { return nullptr; }
    virtual std::shared_ptr<VerifierReferenceString> get_verifier_crs() { return nullptr; }
};

} // namespace proof_system
