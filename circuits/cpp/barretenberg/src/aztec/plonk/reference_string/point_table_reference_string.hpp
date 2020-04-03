/**
 * Create reference strings given a buffer containing point table formatted monomials.
 * Does not take ownership of the given buffer.
 */
#pragma once
#include "mem_reference_string.hpp"

namespace barretenberg {
namespace pairing {
struct miller_lines;
}
} // namespace barretenberg

namespace waffle {

using namespace barretenberg;

class PointTableReferenceString : public ProverReferenceString {
  public:
    PointTableReferenceString(g1::affine_element* monomials);
    ~PointTableReferenceString();

    g1::affine_element* get_monomials() { return monomials_; }

  private:
    g1::affine_element* monomials_;
};

class PointTableReferenceStringFactory : public ReferenceStringFactory {
  public:
    PointTableReferenceStringFactory(g1::affine_element* point_table_buffer, size_t num_points, char const* g2x)
        : point_table_buffer_(point_table_buffer)
        , num_points_(num_points)
        , g2x_(g2x)
    {}

    std::shared_ptr<ProverReferenceString> get_prover_crs(size_t degree)
    {
        ASSERT(degree <= num_points_);
        return std::make_shared<PointTableReferenceString>(point_table_buffer_);
    }

    std::shared_ptr<VerifierReferenceString> get_verifier_crs()
    {
        return std::make_shared<VerifierMemReferenceString>(g2x_);
    }

  private:
    g1::affine_element* point_table_buffer_;
    size_t num_points_;
    char const* g2x_;
};

} // namespace waffle
