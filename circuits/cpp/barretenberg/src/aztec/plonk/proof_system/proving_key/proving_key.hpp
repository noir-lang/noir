#pragma once
#include <ecc/curves/bn254/scalar_multiplication/runtime_states.hpp>
#include <map>
#include <plonk/reference_string/reference_string.hpp>
#include <polynomials/evaluation_domain.hpp>
#include <polynomials/polynomial.hpp>

namespace waffle {

struct proving_key {
  public:
    proving_key(const size_t num_gates, const size_t num_inputs, std::shared_ptr<ProverReferenceString> const& crs);

    proving_key(const proving_key& other);

    proving_key(proving_key&& other);

    proving_key(std::ostream& is, std::string const& crs_path);

    proving_key& operator=(proving_key&& other);

    void reset();

    void write(std::ostream& os);

    size_t n;
    size_t num_public_inputs;

    std::map<std::string, barretenberg::polynomial> constraint_selectors;
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

    barretenberg::scalar_multiplication::unsafe_pippenger_runtime_state pippenger_runtime_state;

    size_t opening_poly_challenge_index;
    size_t shifted_opening_poly_challenge_index;
    static constexpr size_t min_thread_block = 4UL;
};
} // namespace waffle