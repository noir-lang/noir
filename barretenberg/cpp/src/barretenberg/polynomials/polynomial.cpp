#include "polynomial.hpp"
#include "barretenberg/common/assert.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/common/thread.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"
#include "polynomial_arithmetic.hpp"
#include <cstddef>
#include <fcntl.h>
#include <list>
#include <memory>
#include <mutex>
#include <sys/stat.h>
#include <unordered_map>
#include <utility>

namespace bb {

// NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
template <typename Fr> std::shared_ptr<Fr[]> _allocate_aligned_memory(size_t n_elements)
{
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
    return std::static_pointer_cast<Fr[]>(get_mem_slab(sizeof(Fr) * n_elements));
}

template <typename Fr> void Polynomial<Fr>::allocate_backing_memory(size_t size, size_t virtual_size)
{
    coefficients_ = SharedShiftedVirtualZeroesArray<Fr>{
        size,         /* actual memory size */
        virtual_size, /* virtual size, i.e. until what size do we conceptually have zeroes */
        0,            /* shift, initially 0 */
        _allocate_aligned_memory<Fr>(size + MAXIMUM_COEFFICIENT_SHIFT)
        /* Our backing memory, since shift is 0 it is equal to our memory size.
         * We add one to the size here to allow for an efficient shift by 1 that retains size. */
    };
    // We need to zero the extra padding memory that we reserve for shifts.
    // We do this here as generally code that does not zero memory and then
    // later initializes it won't generally also initialize the padding.
    for (size_t i = 0; i < MAXIMUM_COEFFICIENT_SHIFT; i++) {
        data()[size + i] = Fr{};
    }
}

/**
 * Constructors / Destructors
 **/

/**
 * @brief Initialize a Polynomial to size 'size', zeroing memory.
 *
 * @param size The size of the polynomial.
 */
template <typename Fr> Polynomial<Fr>::Polynomial(size_t size, size_t virtual_size)
{
    allocate_backing_memory(size, virtual_size);
    memset(static_cast<void*>(coefficients_.data()), 0, sizeof(Fr) * size);
}

/**
 * @brief Initialize a Polynomial to size 'size'.
 * Important: This does NOT zero memory.
 *
 * @param size The initial size of the polynomial.
 * @param flag Signals that we do not zero memory.
 */
template <typename Fr> Polynomial<Fr>::Polynomial(size_t size, size_t virtual_size, DontZeroMemory flag)
{
    // Flag is unused, but we don't memset 0 if passed.
    (void)flag;
    allocate_backing_memory(size, virtual_size);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(const Polynomial<Fr>& other)
    : Polynomial<Fr>(other, other.size())
{}

// fully copying "expensive" constructor
template <typename Fr> Polynomial<Fr>::Polynomial(const Polynomial<Fr>& other, const size_t target_size)
{
    allocate_backing_memory(std::max(target_size, other.size()), other.virtual_size());

    memcpy(static_cast<void*>(coefficients_.data()),
           static_cast<const void*>(other.coefficients_.data()),
           sizeof(Fr) * other.size());
    zero_memory_beyond(other.size());
}

// interpolation constructor
template <typename Fr>
Polynomial<Fr>::Polynomial(std::span<const Fr> interpolation_points,
                           std::span<const Fr> evaluations,
                           size_t virtual_size)
    : Polynomial(interpolation_points.size(), virtual_size)
{
    ASSERT(coefficients_.size_ > 0);

    polynomial_arithmetic::compute_efficient_interpolation(
        evaluations.data(), coefficients_.data(), interpolation_points.data(), coefficients_.size_);
}

template <typename Fr> Polynomial<Fr>::Polynomial(std::span<const Fr> coefficients, size_t virtual_size)
{
    allocate_backing_memory(coefficients.size(), virtual_size);

    memcpy(static_cast<void*>(data()), static_cast<const void*>(coefficients.data()), sizeof(Fr) * coefficients.size());
}

// Assignments

// full copy "expensive" assignment
template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(const Polynomial<Fr>& other)
{
    if (this == &other) {
        return *this;
    }
    allocate_backing_memory(other.coefficients_.size_, other.coefficients_.virtual_size_);
    memcpy(static_cast<void*>(coefficients_.data()),
           static_cast<const void*>(other.coefficients_.data()),
           sizeof(Fr) * other.coefficients_.size_);
    return *this;
}

template <typename Fr> Polynomial<Fr> Polynomial<Fr>::share() const
{
    Polynomial p;
    p.coefficients_ = coefficients_;
    return p;
}

template <typename Fr> bool Polynomial<Fr>::operator==(Polynomial const& rhs) const
{
    // If either is empty, both must be
    if (is_empty() || rhs.is_empty()) {
        return is_empty() && rhs.is_empty();
    }
    // Size must agree
    if (virtual_size() != rhs.virtual_size()) {
        return false;
    }
    // Each coefficient must agree
    for (size_t i = 0; i < std::max(size(), rhs.size()); i++) {
        if (coefficients_.get(i) != rhs.coefficients_.get(i)) {
            return false;
        }
    }
    return true;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator+=(std::span<const Fr> other)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    size_t num_threads = calculate_num_threads(other_size);
    size_t range_per_thread = other_size / num_threads;
    size_t leftovers = other_size - (range_per_thread * num_threads);
    parallel_for(num_threads, [&](size_t j) {
        size_t offset = j * range_per_thread;
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            coefficients_.data()[i] += other[i];
        }
    });

    return *this;
}

template <typename Fr> Fr Polynomial<Fr>::evaluate(const Fr& z, const size_t target_size) const
{
    return polynomial_arithmetic::evaluate(data(), z, target_size);
}

template <typename Fr> Fr Polynomial<Fr>::evaluate(const Fr& z) const
{
    return polynomial_arithmetic::evaluate(data(), z, size());
}

template <typename Fr> Fr Polynomial<Fr>::evaluate_mle(std::span<const Fr> evaluation_points, bool shift) const
{
    const size_t m = evaluation_points.size();

    // To simplify handling of edge cases, we assume that size_ is always a power of 2
    ASSERT(size() == static_cast<size_t>(1 << m));

    // we do m rounds l = 0,...,m-1.
    // in round l, n_l is the size of the buffer containing the Polynomial partially evaluated
    // at u₀,..., u_l.
    // in round 0, this is half the size of n
    size_t n_l = 1 << (m - 1);

    // temporary buffer of half the size of the Polynomial
    // TODO(AD): Make this a Polynomial with DontZeroMemory::FLAG
    auto tmp_ptr = _allocate_aligned_memory<Fr>(sizeof(Fr) * n_l);
    auto tmp = tmp_ptr.get();

    const Fr* prev = data();
    if (shift) {
        ASSERT(prev[0] == Fr::zero());
        prev++;
    }

    Fr u_l = evaluation_points[0];
    for (size_t i = 0; i < n_l; ++i) {
        // curr[i] = (Fr(1) - u_l) * prev[i << 1] + u_l * prev[(i << 1) + 1];
        tmp[i] = prev[i << 1] + u_l * (prev[(i << 1) + 1] - prev[i << 1]);
    }
    // partially evaluate the m-1 remaining points
    for (size_t l = 1; l < m; ++l) {
        n_l = 1 << (m - l - 1);
        u_l = evaluation_points[l];
        for (size_t i = 0; i < n_l; ++i) {
            tmp[i] = tmp[i << 1] + u_l * (tmp[(i << 1) + 1] - tmp[i << 1]);
        }
    }
    Fr result = tmp[0];
    return result;
}

template <typename Fr> Polynomial<Fr> Polynomial<Fr>::partial_evaluate_mle(std::span<const Fr> evaluation_points) const
{
    // Get size of partial evaluation point u = (u_0,...,u_{m-1})
    const size_t m = evaluation_points.size();

    // Assert that the size of the Polynomial being evaluated is a power of 2 greater than (1 << m)
    ASSERT(numeric::is_power_of_two(size()));
    ASSERT(size() >= static_cast<size_t>(1 << m));
    size_t n = numeric::get_msb(size());

    // Partial evaluation is done in m rounds l = 0,...,m-1. At the end of round l, the Polynomial has been
    // partially evaluated at u_{m-l-1}, ..., u_{m-1} in variables X_{n-l-1}, ..., X_{n-1}. The size of this
    // Polynomial is n_l.
    size_t n_l = 1 << (n - 1);

    // Temporary buffer of half the size of the Polynomial
    Polynomial<Fr> intermediate(n_l, n_l, DontZeroMemory::FLAG);

    // Evaluate variable X_{n-1} at u_{m-1}
    Fr u_l = evaluation_points[m - 1];

    for (size_t i = 0; i < n_l; i++) {
        // Initiate our intermediate results using this Polynomial.
        intermediate[i] = get(i) + u_l * (get(i + n_l) - get(i));
    }
    // Evaluate m-1 variables X_{n-l-1}, ..., X_{n-2} at m-1 remaining values u_0,...,u_{m-2})
    for (size_t l = 1; l < m; ++l) {
        n_l = 1 << (n - l - 1);
        u_l = evaluation_points[m - l - 1];
        for (size_t i = 0; i < n_l; ++i) {
            intermediate[i] += u_l * (intermediate[i + n_l] - intermediate[i]);
        }
    }

    // Construct resulting Polynomial g(X_0,…,X_{n-m-1})) = p(X_0,…,X_{n-m-1},u_0,...u_{m-1}) from buffer
    Polynomial<Fr> result(n_l, n_l, DontZeroMemory::FLAG);
    for (size_t idx = 0; idx < n_l; ++idx) {
        result[idx] = intermediate[idx];
    }

    return result;
}

template <typename Fr>
Fr Polynomial<Fr>::compute_kate_opening_coefficients(const Fr& z)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    return polynomial_arithmetic::compute_kate_opening_coefficients(data(), data(), z, size());
}

template <typename Fr>
Fr Polynomial<Fr>::compute_barycentric_evaluation(const Fr& z, const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    return polynomial_arithmetic::compute_barycentric_evaluation(data(), domain.size, z, domain);
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator-=(std::span<const Fr> other)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    size_t num_threads = calculate_num_threads(other_size);
    size_t range_per_thread = other_size / num_threads;
    size_t leftovers = other_size - (range_per_thread * num_threads);
    parallel_for(num_threads, [&](size_t j) {
        size_t offset = j * range_per_thread;
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            coefficients_.data()[i] -= other[i];
        }
    });

    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator*=(const Fr scaling_factor)
{
    ASSERT(in_place_operation_viable());

    size_t num_threads = calculate_num_threads(size());
    size_t range_per_thread = size() / num_threads;
    size_t leftovers = size() - (range_per_thread * num_threads);
    parallel_for(num_threads, [&](size_t j) {
        size_t offset = j * range_per_thread;
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            coefficients_.data()[i] *= scaling_factor;
        }
    });

    return *this;
}

template <typename Fr> void Polynomial<Fr>::add_scaled(std::span<const Fr> other, Fr scaling_factor)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    size_t num_threads = calculate_num_threads(other_size);
    size_t range_per_thread = other_size / num_threads;
    size_t leftovers = other_size - (range_per_thread * num_threads);
    parallel_for(num_threads, [&](size_t j) {
        size_t offset = j * range_per_thread;
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            data()[i] += scaling_factor * other[i];
        }
    });
}

/**
 * @brief Returns a Polynomial the left-shift of self.
 *
 * @details If the n coefficients of self are (0, a₁, …, aₙ₋₁),
 * we returns the view of the n-1 coefficients (a₁, …, aₙ₋₁).
 */
template <typename Fr> Polynomial<Fr> Polynomial<Fr>::shifted() const
{
    ASSERT(data()[0].is_zero());
    ASSERT(size() > 0);
    ASSERT(data()[size()].is_zero()); // relies on MAXIMUM_COEFFICIENT_SHIFT >= 1
    Polynomial result;
    result.coefficients_ = coefficients_;
    result.coefficients_.shift_ += 1;
    // We only expect to shift by MAXIMUM_COEFFICIENT_SHIFT
    ASSERT(result.coefficients_.shift_ <= MAXIMUM_COEFFICIENT_SHIFT);
    return result;
}

/**
 * @brief sets a block of memory to all zeroes
 * Used, for example, when one Polynomioal is instantiated from another one with size_>= other.size_.
 */
template <typename Fr> void Polynomial<Fr>::zero_memory_beyond(const size_t start_position)
{
    size_t end = size();
    ASSERT(end >= start_position);

    size_t delta = end - start_position;
    if (delta > 0) {
        memset(static_cast<void*>(&data()[start_position]), 0, sizeof(Fr) * delta);
    }
}

template class Polynomial<bb::fr>;
template class Polynomial<grumpkin::fr>;

} // namespace bb