#pragma once
#include <map>
#include <polynomials/evaluation_domain.hpp>
#include <polynomials/polynomial.hpp>
#include <plonk/reference_string/reference_string.hpp>

namespace waffle
{
struct proving_key
{
    public:
    proving_key(const size_t num_gates, const size_t num_inputs, std::string const& crs_path);

    proving_key(const proving_key& other);

    proving_key(proving_key&& other);

    proving_key& operator=(proving_key&& other);

    void reset();

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

    ReferenceString reference_string;

    barretenberg::polynomial z;
    barretenberg::polynomial z_fft;
    barretenberg::polynomial lagrange_1;
    barretenberg::polynomial opening_poly;
    barretenberg::polynomial shifted_opening_poly;
    barretenberg::polynomial linear_poly;

    barretenberg::polynomial quotient_mid;
    barretenberg::polynomial quotient_large;
    static constexpr size_t min_thread_block = 4UL;
};
}