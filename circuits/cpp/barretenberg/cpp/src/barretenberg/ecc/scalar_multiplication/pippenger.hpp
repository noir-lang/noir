#pragma once
#include "./scalar_multiplication.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/max_threads.hpp"

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace barretenberg {
namespace scalar_multiplication {

inline size_t point_table_size(size_t num_points)
{
#ifndef NO_MULTITHREADING
    const size_t num_threads = max_threads::compute_num_threads();
#else
    const size_t num_threads = 1;
#endif
    const size_t prefetch_overflow = 16 * num_threads;

    return 2 * num_points + prefetch_overflow;
}

template <typename T> inline size_t point_table_buf_size(size_t num_points)
{
    return sizeof(T) * point_table_size(num_points);
}

template <typename T> inline T* point_table_alloc(size_t num_points)
{
    return (T*)aligned_alloc(64, point_table_buf_size<T>(num_points));
}

template <typename Curve> class Pippenger {
  public:
    using ScalarField = typename Curve::ScalarField;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    /**
     * Expects points to be buffer of size as per point_table_size().
     * It expects the crs to start at points[1], and it fills in affine_one at points[0].
     * The crs undergoes a byteswap, and then the point table is generated.
     */
    Pippenger(AffineElement* points, size_t num_points);

    Pippenger(uint8_t const* points, size_t num_points);

    Pippenger(std::string const& path, size_t num_points);

    ~Pippenger();

    Element pippenger_unsafe(ScalarField* scalars, size_t from, size_t range);

    AffineElement* get_point_table() const { return monomials_; }

    size_t get_num_points() const { return num_points_; }

  private:
    AffineElement* monomials_;
    size_t num_points_;
};

extern template class Pippenger<curve::BN254>;
extern template class Pippenger<curve::Grumpkin>;

} // namespace scalar_multiplication
} // namespace barretenberg
