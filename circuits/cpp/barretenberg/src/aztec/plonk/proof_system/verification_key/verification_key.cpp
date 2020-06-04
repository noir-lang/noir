#include "verification_key.hpp"

namespace waffle {

verification_key::verification_key(const size_t num_gates,
                                   const size_t num_inputs,
                                   std::shared_ptr<VerifierReferenceString> const& crs)
    : n(num_gates)
    , num_public_inputs(num_inputs)
    , domain(n)
    , reference_string(crs)
{}

verification_key::verification_key(verification_key_data&& data, std::shared_ptr<VerifierReferenceString> const& crs)
    : n(data.n)
    , num_public_inputs(data.num_public_inputs)
    , domain(n)
    , reference_string(crs)
    , constraint_selectors(std::move(data.constraint_selectors))
    , permutation_selectors(std::move(data.permutation_selectors))
{}

verification_key::verification_key(const verification_key& other)
    : n(other.n)
    , num_public_inputs(other.num_public_inputs)
    , domain(other.domain)
    , reference_string(other.reference_string)
    , constraint_selectors(other.constraint_selectors)
    , permutation_selectors(other.permutation_selectors)
    , polynomial_manifest(other.polynomial_manifest)
{}

verification_key::verification_key(verification_key&& other)
    : n(other.n)
    , num_public_inputs(other.num_public_inputs)
    , domain(other.domain)
    , reference_string(other.reference_string)
    , constraint_selectors(other.constraint_selectors)
    , permutation_selectors(other.permutation_selectors)
    , polynomial_manifest(other.polynomial_manifest)
{}

verification_key& verification_key::operator=(verification_key&& other)
{
    n = other.n;
    num_public_inputs = other.num_public_inputs;
    reference_string = std::move(other.reference_string);
    constraint_selectors = std::move(other.constraint_selectors);
    permutation_selectors = std::move(other.permutation_selectors);
    polynomial_manifest = std::move(other.polynomial_manifest);
    return *this;
}
} // namespace waffle