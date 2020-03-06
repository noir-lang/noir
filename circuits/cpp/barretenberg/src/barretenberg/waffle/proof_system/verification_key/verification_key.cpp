#include "./verification_key.hpp"

namespace waffle {

verification_key::verification_key(const size_t num_gates, const size_t num_inputs, std::string const& crs_path)
    : n(num_gates)
    , num_public_inputs(num_inputs)
    , domain(n)
    , reference_string(n, crs_path)
{}

verification_key::verification_key(const verification_key& other)
    : n(other.n)
    , num_public_inputs(other.num_public_inputs)
    , domain(other.domain)
    , reference_string(other.reference_string)
    , constraint_selectors(other.constraint_selectors)
    , permutation_selectors(other.permutation_selectors)
{}

verification_key::verification_key(verification_key&& other)
    : n(other.n)
    , num_public_inputs(other.num_public_inputs)
    , domain(other.domain)
    , reference_string(other.reference_string)
    , constraint_selectors(other.constraint_selectors)
    , permutation_selectors(other.permutation_selectors)
{}

verification_key& verification_key::operator=(verification_key&& other)
{
    n = other.n;
    num_public_inputs = other.num_public_inputs;
    reference_string = std::move(other.reference_string);
    constraint_selectors = std::move(other.constraint_selectors);
    permutation_selectors = std::move(other.permutation_selectors);
    return *this;
}
} // namespace waffle