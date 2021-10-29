#pragma once
#include <ecc/curves/bn254/scalar_multiplication/runtime_states.hpp>
#include <map>
#include <plonk/reference_string/reference_string.hpp>
#include <polynomials/evaluation_domain.hpp>
#include <polynomials/polynomial.hpp>

#include "../types/polynomial_manifest.hpp"

namespace waffle {

struct proving_key_data {
    uint32_t composer_type;
    uint32_t n;
    uint32_t num_public_inputs;
    bool contains_recursive_proof;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    std::map<std::string, barretenberg::polynomial> constraint_selectors;
    std::map<std::string, barretenberg::polynomial> constraint_selector_ffts;
    std::map<std::string, barretenberg::polynomial> permutation_selectors;
    std::map<std::string, barretenberg::polynomial> permutation_selectors_lagrange_base;
    std::map<std::string, barretenberg::polynomial> permutation_selector_ffts;
};

inline bool operator==(proving_key_data const& lhs, proving_key_data const& rhs)
{
    return lhs.composer_type == rhs.composer_type && lhs.n == rhs.n && lhs.num_public_inputs == rhs.num_public_inputs &&
           lhs.constraint_selectors == rhs.constraint_selectors &&
           lhs.constraint_selector_ffts == rhs.constraint_selector_ffts &&
           lhs.permutation_selectors == rhs.permutation_selectors &&
           lhs.permutation_selectors_lagrange_base == rhs.permutation_selectors_lagrange_base &&
           lhs.permutation_selector_ffts == rhs.permutation_selector_ffts &&
           lhs.contains_recursive_proof == rhs.contains_recursive_proof &&
           lhs.recursive_proof_public_input_indices == rhs.recursive_proof_public_input_indices;
}

struct proving_key {
  public:
    enum LookupType {
        NONE,
        ABSOLUTE_LOOKUP,
        RELATIVE_LOOKUP,
    };

    proving_key(proving_key_data&& data, std::shared_ptr<ProverReferenceString> const& crs);

    proving_key(const size_t num_gates, const size_t num_inputs, std::shared_ptr<ProverReferenceString> const& crs);

    proving_key(const proving_key& other);

    proving_key(proving_key&& other);

    proving_key(std::ostream& is, std::string const& crs_path);

    proving_key& operator=(proving_key&& other);

    void reset();

    void init();

    uint32_t composer_type;
    size_t n;
    size_t num_public_inputs;

    std::map<std::string, barretenberg::polynomial> constraint_selectors;
    std::map<std::string, barretenberg::polynomial> constraint_selectors_lagrange_base;
    std::map<std::string, barretenberg::polynomial> constraint_selector_ffts;

    std::map<std::string, barretenberg::polynomial> permutation_selectors;
    std::map<std::string, barretenberg::polynomial> permutation_selectors_lagrange_base;
    std::map<std::string, barretenberg::polynomial> permutation_selector_ffts;

    std::map<std::string, barretenberg::polynomial> wire_ffts;

    barretenberg::evaluation_domain small_domain;
    barretenberg::evaluation_domain mid_domain;
    barretenberg::evaluation_domain large_domain;

    std::shared_ptr<ProverReferenceString> reference_string;

    barretenberg::polynomial lagrange_1;
    barretenberg::polynomial opening_poly;
    barretenberg::polynomial shifted_opening_poly;
    barretenberg::polynomial linear_poly;

    barretenberg::polynomial quotient_mid;
    barretenberg::polynomial quotient_large;

    barretenberg::scalar_multiplication::pippenger_runtime_state pippenger_runtime_state;

    std::vector<PolynomialDescriptor> polynomial_manifest;

    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    static constexpr size_t min_thread_block = 4UL;
};

} // namespace waffle
