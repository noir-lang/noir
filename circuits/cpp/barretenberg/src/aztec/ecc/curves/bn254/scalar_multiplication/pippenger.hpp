#pragma once
#include "./scalar_multiplication.hpp"
#include <common/mem.hpp>

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace barretenberg {
namespace scalar_multiplication {

inline size_t point_table_size(size_t num_points)
{
#ifndef NO_MULTITHREADING
    const size_t num_threads = static_cast<size_t>(omp_get_max_threads());
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

class Pippenger {
  public:
    Pippenger(uint8_t const* points, size_t num_points);

    Pippenger(std::string const& path, size_t num_points);

    ~Pippenger();

    g1::element pippenger_unsafe(fr* scalars, size_t from, size_t range);

    g1::affine_element* get_point_table() const { return monomials_; }

    size_t get_num_points() const { return num_points_; }

  private:
    g1::affine_element* monomials_;
    size_t num_points_;
};

} // namespace scalar_multiplication
} // namespace barretenberg