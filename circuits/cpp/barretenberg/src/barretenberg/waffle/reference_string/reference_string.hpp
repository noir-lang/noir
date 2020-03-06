#pragma once

#include <cstddef>

#include "../../curves/bn254/g1.hpp"
#include "../../curves/bn254/g2.hpp"

namespace barretenberg
{
namespace pairing
{
struct miller_lines;
}
}

namespace waffle
{
class VerifierReferenceString
{
public:

    VerifierReferenceString();
    VerifierReferenceString(const size_t num_points, std::string const& path);
    VerifierReferenceString(const VerifierReferenceString &other);
    VerifierReferenceString(VerifierReferenceString &&other);

    VerifierReferenceString & operator=(const VerifierReferenceString& other);
    VerifierReferenceString & operator=(VerifierReferenceString &&other);

    ~VerifierReferenceString();

    barretenberg::g2::affine_element g2_x;

    barretenberg::pairing::miller_lines *precomputed_g2_lines;

    size_t degree;
};

class ReferenceString
{
public:

    ReferenceString();
    ReferenceString(const size_t num_points, std::string const& path);
    ReferenceString(const ReferenceString &other);
    ReferenceString(ReferenceString &&other);

    ReferenceString & operator=(const ReferenceString& other);
    ReferenceString & operator=(ReferenceString &&other);

    ~ReferenceString();

    ReferenceString get_verifier_reference_string() const;

    barretenberg::g1::affine_element *monomials;
    barretenberg::g2::affine_element g2_x;

    barretenberg::pairing::miller_lines *precomputed_g2_lines;

    size_t degree;
};
}
