#pragma once
#include <ecc/curves/bn254/scalar_multiplication/runtime_states.hpp>
#include <map>
#include <polynomials/evaluation_domain.hpp>
#include <polynomials/polynomial.hpp>

#include <srs/reference_string/reference_string.hpp>
#include <plonk/proof_system/constants.hpp>
#include <proof_system/types/polynomial_manifest.hpp>
#include <unordered_map>

#include "../polynomial_cache/polynomial_cache.hpp"

namespace waffle {

struct proving_key_data {
    uint32_t composer_type;
    uint32_t n;
    uint32_t num_public_inputs;
    bool contains_recursive_proof;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    std::vector<uint32_t> memory_records;
    PolynomialCache polynomial_cache;
};

struct proving_key {
  public:
    enum LookupType {
        NONE,
        ABSOLUTE_LOOKUP,
        RELATIVE_LOOKUP,
    };

    proving_key(proving_key_data&& data, std::shared_ptr<ProverReferenceString> const& crs);

    proving_key(const size_t num_gates,
                const size_t num_inputs,
                std::shared_ptr<ProverReferenceString> const& crs,
                waffle::ComposerType type);

    proving_key(std::ostream& is, std::string const& crs_path);

    void init();

    uint32_t composer_type;
    size_t n;
    size_t log_n;
    size_t num_public_inputs;
    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    std::vector<uint32_t> memory_records; // Used by UltraComposer only; for ROM.
    // Note: low-memory prover functionality can be achieved by uncommenting the lines below
    // which allow the polynomial cache to write polynomials to file as necessary. Similar
    // lines must also be uncommented in constructor.
    // #ifdef __wasm__
    //     PolynomialStoreWasm underlying_store;
    // #endif
    PolynomialCache polynomial_cache;

    barretenberg::evaluation_domain small_domain;
    barretenberg::evaluation_domain large_domain;

    // We are keeping only one reference string which would be monomial srs if we use the Kate based PLONK,
    // and Lagrange srs if we use the SHPLONK based PLONK.
    std::shared_ptr<ProverReferenceString> reference_string;

    barretenberg::polynomial quotient_polynomial_parts[NUM_QUOTIENT_PARTS];

    barretenberg::scalar_multiplication::pippenger_runtime_state pippenger_runtime_state;

    PolynomialManifest polynomial_manifest;

    static constexpr size_t min_thread_block = 4UL;
};

} // namespace waffle
