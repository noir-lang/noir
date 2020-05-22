#pragma once
#include <ecc/curves/bn254/scalar_multiplication/runtime_states.hpp>
#include <map>
#include <plonk/reference_string/reference_string.hpp>
#include <polynomials/evaluation_domain.hpp>
#include <polynomials/polynomial.hpp>

namespace waffle {

struct proving_key_data {
    uint32_t n;
    uint32_t num_public_inputs;
    std::map<std::string, barretenberg::polynomial> constraint_selectors;
    std::map<std::string, barretenberg::polynomial> constraint_selector_ffts;
    std::map<std::string, barretenberg::polynomial> permutation_selectors;
    std::map<std::string, barretenberg::polynomial> permutation_selectors_lagrange_base;
    std::map<std::string, barretenberg::polynomial> permutation_selector_ffts;
};

inline bool operator==(proving_key_data const& lhs, proving_key_data const& rhs)
{
    return lhs.n == rhs.n && lhs.num_public_inputs == rhs.num_public_inputs &&
           lhs.constraint_selectors == rhs.constraint_selectors &&
           lhs.constraint_selector_ffts == rhs.constraint_selector_ffts &&
           lhs.permutation_selectors == rhs.permutation_selectors &&
           lhs.permutation_selectors_lagrange_base == rhs.permutation_selectors_lagrange_base &&
           lhs.permutation_selector_ffts == rhs.permutation_selector_ffts;
}

template <typename B> inline void read(B& buf, proving_key_data& key)
{
    ::read(buf, key.n);
    ::read(buf, key.num_public_inputs);
    read(buf, key.constraint_selectors);
    read(buf, key.constraint_selector_ffts);
    read(buf, key.permutation_selectors);
    read(buf, key.permutation_selectors_lagrange_base);
    read(buf, key.permutation_selector_ffts);
}

template <typename B> inline void write(B& buf, proving_key_data const& key)
{
    ::write(buf, key.n);
    ::write(buf, key.num_public_inputs);
    write(buf, key.constraint_selectors);
    write(buf, key.constraint_selector_ffts);
    write(buf, key.permutation_selectors);
    write(buf, key.permutation_selectors_lagrange_base);
    write(buf, key.permutation_selector_ffts);
}

struct proving_key {
  public:
    proving_key(proving_key_data&& data, std::shared_ptr<ProverReferenceString> const& crs);

    proving_key(const size_t num_gates, const size_t num_inputs, std::shared_ptr<ProverReferenceString> const& crs);

    proving_key(const proving_key& other);

    proving_key(proving_key&& other);

    proving_key(std::ostream& is, std::string const& crs_path);

    proving_key& operator=(proving_key&& other);

    void reset();

    void init();

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

    barretenberg::scalar_multiplication::pippenger_runtime_state pippenger_runtime_state;

    size_t opening_poly_challenge_index;
    size_t shifted_opening_poly_challenge_index;
    static constexpr size_t min_thread_block = 4UL;
};

template <typename B> inline void write(B& buf, proving_key const& key_)
{
    auto key = const_cast<proving_key&>(key_);
    proving_key_data data = {
        static_cast<uint32_t>(key.n),
        static_cast<uint32_t>(key.num_public_inputs),
        std::move(key.constraint_selectors),
        std::move(key.constraint_selector_ffts),
        std::move(key.permutation_selectors),
        std::move(key.permutation_selectors_lagrange_base),
        std::move(key.permutation_selector_ffts),
    };
    write(buf, data);
    key.constraint_selectors = std::move(data.constraint_selectors);
    key.constraint_selector_ffts = std::move(data.constraint_selector_ffts);
    key.permutation_selectors = std::move(data.permutation_selectors);
    key.permutation_selectors_lagrange_base = std::move(data.permutation_selectors_lagrange_base);
    key.permutation_selector_ffts = std::move(data.permutation_selector_ffts);
}

} // namespace waffle