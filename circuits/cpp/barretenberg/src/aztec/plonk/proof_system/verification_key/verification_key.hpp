#pragma once
#include <map>
#include <plonk/reference_string/reference_string.hpp>
#include <polynomials/evaluation_domain.hpp>

namespace waffle {

struct verification_key {
    verification_key(const size_t num_gates,
                     const size_t num_inputs,
                     std::shared_ptr<VerifierReferenceString> const& crs);
    verification_key(const verification_key& other);
    verification_key(verification_key&& other);
    verification_key& operator=(verification_key&& other);

    ~verification_key() = default;
    size_t n;
    size_t num_public_inputs;

    barretenberg::evaluation_domain domain;

    std::shared_ptr<VerifierReferenceString> reference_string;

    std::map<std::string, barretenberg::g1::affine_element> constraint_selectors;

    std::map<std::string, barretenberg::g1::affine_element> permutation_selectors;

    size_t program_width = 3;
};
} // namespace waffle