#pragma once
#include <map>
#include <unordered_map>

#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/scalar_multiplication/runtime_states.hpp"
#include "barretenberg/plonk/proof_system/constants.hpp"
#include "barretenberg/plonk/proof_system/types/polynomial_manifest.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"

#ifdef __wasm__
#include "barretenberg/proof_system/polynomial_store/polynomial_store_cache.hpp"
// #include "barretenberg/proof_system/polynomial_store/polynomial_store_wasm.hpp"
#else
#include "barretenberg/proof_system/polynomial_store/polynomial_store.hpp"
#endif

namespace bb::plonk {

struct proving_key_data {
    uint32_t circuit_type;
    uint32_t circuit_size;
    uint32_t num_public_inputs;
    bool contains_recursive_proof;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    std::vector<uint32_t> memory_read_records;
    std::vector<uint32_t> memory_write_records;
#ifdef __wasm__
    PolynomialStoreCache polynomial_store;
    // PolynomialStoreWasm<bb::fr> polynomial_store;
#else
    PolynomialStore<bb::fr> polynomial_store;
#endif
};

struct proving_key {
  public:
    enum LookupType {
        NONE,
        ABSOLUTE_LOOKUP,
        RELATIVE_LOOKUP,
    };

    proving_key(proving_key_data&& data, std::shared_ptr<bb::srs::factories::ProverCrs<curve::BN254>> const& crs);

    proving_key(const size_t num_gates,
                const size_t num_inputs,
                std::shared_ptr<bb::srs::factories::ProverCrs<curve::BN254>> const& crs,
                CircuitType type = CircuitType::UNDEFINED);

    proving_key(std::ostream& is, std::string const& crs_path);

    void init();

    CircuitType circuit_type;
    size_t circuit_size;
    size_t log_circuit_size;
    size_t num_public_inputs;
    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    std::vector<uint32_t> memory_read_records;  // Used by UltraPlonkComposer only; for ROM, RAM reads.
    std::vector<uint32_t> memory_write_records; // Used by UltraPlonkComposer only, for RAM writes.

#ifdef __wasm__
    PolynomialStoreCache polynomial_store;
    // PolynomialStoreWasm<bb::fr> polynomial_store;
#else
    PolynomialStore<bb::fr> polynomial_store;
#endif

    bb::evaluation_domain small_domain;
    bb::evaluation_domain large_domain;

    // The reference_string object contains the monomial SRS. We can access it using:
    // Monomial SRS: reference_string->get_monomial_points()
    std::shared_ptr<bb::srs::factories::ProverCrs<curve::BN254>> reference_string;

    bb::polynomial quotient_polynomial_parts[plonk::NUM_QUOTIENT_PARTS];

    PolynomialManifest polynomial_manifest;

    static constexpr size_t min_thread_block = 4UL;
};

} // namespace bb::plonk
