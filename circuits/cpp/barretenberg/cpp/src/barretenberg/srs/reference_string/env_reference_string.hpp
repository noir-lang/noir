/**
 * Create reference strings given an environment that implements env/crs.hpp.
 * Usable in both native and WASM code, but particularly useful for accessing
 * reference strings from a WASM context.
 * In a native context, the implementations assume a hard-coded string path.
 * For that reason, ideally this is only used in c-bind functions to maintain flexibility.
 */
#pragma once
#include "barretenberg/srs/reference_string/mem_reference_string.hpp"
#include "reference_string.hpp"

#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/bn254/g2.hpp"
#include "barretenberg/ecc/curves/bn254/scalar_multiplication/pippenger.hpp"

#include "barretenberg/env/crs.hpp"

#include <utility>
#include <cstddef>
namespace proof_system {

class EnvReferenceString : public ProverReferenceString {
  public:
    EnvReferenceString(const size_t num_points)
        : num_points(num_points)
        , pippenger_(env_load_prover_crs(num_points), num_points)
    {}

    g1::affine_element* get_monomial_points() override { return pippenger_.get_point_table(); }

    size_t get_monomial_size() const override { return num_points; }

  private:
    size_t num_points;
    scalar_multiplication::Pippenger pippenger_;
};

class EnvReferenceStringFactory : public ReferenceStringFactory {
  public:
    std::shared_ptr<ProverReferenceString> get_prover_crs(size_t degree) override
    {
        return std::make_shared<EnvReferenceString>(degree);
    }

    std::shared_ptr<VerifierReferenceString> get_verifier_crs() override
    {
        return std::make_shared<VerifierMemReferenceString>(env_load_verifier_crs());
    }
};

} // namespace proof_system
