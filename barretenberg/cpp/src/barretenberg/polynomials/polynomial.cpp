#include "polynomial.hpp"
#include "barretenberg/common/assert.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/common/thread.hpp"
#include "barretenberg/common/thread_utils.hpp"
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
template <typename Fr> std::shared_ptr<Fr[]> _allocate_aligned_memory(const size_t n_elements)
{
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
    return std::static_pointer_cast<Fr[]>(get_mem_slab(sizeof(Fr) * n_elements));
}

template <typename Fr> void Polynomial<Fr>::allocate_backing_memory(size_t n_elements)
{
    size_ = n_elements;
    // capacity() is size_ plus padding for shifted polynomials
    backing_memory_ = _allocate_aligned_memory<Fr>(capacity());
    coefficients_ = backing_memory_.get();
}

/**
 * Constructors / Destructors
 **/

/**
 * @brief Initialize a Polynomial to size 'initial_size', zeroing memory.
 *
 * @param initial_size The initial size of the polynomial.
 */
template <typename Fr> Polynomial<Fr>::Polynomial(size_t initial_size)
{
    allocate_backing_memory(initial_size);
    memset(static_cast<void*>(coefficients_), 0, sizeof(Fr) * capacity());
}

/**
 * @brief Initialize a Polynomial to size 'initial_size'.
 * Important: This does NOT zero memory.
 *
 * @param initial_size The initial size of the polynomial.
 * @param flag Signals that we do not zero memory.
 */
template <typename Fr> Polynomial<Fr>::Polynomial(size_t initial_size, DontZeroMemory flag)
{
    // Flag is unused, but we don't memset 0 if passed.
    (void)flag;
    allocate_backing_memory(initial_size);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(const Polynomial<Fr>& other)
    : Polynomial<Fr>(other, other.size())
{}

// fully copying "expensive" constructor
template <typename Fr> Polynomial<Fr>::Polynomial(const Polynomial<Fr>& other, const size_t target_size)
{
    allocate_backing_memory(std::max(target_size, other.size()));

    memcpy(static_cast<void*>(coefficients_), static_cast<void*>(other.coefficients_), sizeof(Fr) * other.size_);
    zero_memory_beyond(other.size_);
}

// move constructor
template <typename Fr>
Polynomial<Fr>::Polynomial(Polynomial<Fr>&& other) noexcept
    : backing_memory_(std::exchange(other.backing_memory_, nullptr))
    , coefficients_(std::exchange(other.coefficients_, nullptr))
    , size_(std::exchange(other.size_, 0))
{}

// span constructor
template <typename Fr> Polynomial<Fr>::Polynomial(std::span<const Fr> coefficients)
{
    allocate_backing_memory(coefficients.size());
    memcpy(static_cast<void*>(coefficients_),
           static_cast<void const*>(coefficients.data()),
           sizeof(Fr) * coefficients.size());
    zero_memory_beyond(size_);
}

// interpolation constructor
template <typename Fr>
Polynomial<Fr>::Polynomial(std::span<const Fr> interpolation_points, std::span<const Fr> evaluations)
    : Polynomial(interpolation_points.size())
{
    ASSERT(size_ > 0);

    polynomial_arithmetic::compute_efficient_interpolation(
        evaluations.data(), coefficients_, interpolation_points.data(), size_);
}

// Assignments

// full copy "expensive" assignment
template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(const Polynomial<Fr>& other)
{
    if (this == &other) {
        return *this;
    }
    allocate_backing_memory(other.size_);
    memcpy(static_cast<void*>(coefficients_), static_cast<void*>(other.coefficients_), sizeof(Fr) * other.size_);
    zero_memory_beyond(size_);
    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(std::span<const Fr> coefficients) noexcept
{
    // move assign a Polynomial constructed with the span
    *this = Polynomial<Fr>{ coefficients };
    return *this;
}

// #######
template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(Polynomial&& other) noexcept
{
    if (&other == this) {
        return *this;
    }

    // simultaneously set members and clear other
    backing_memory_ = std::exchange(other.backing_memory_, nullptr);
    coefficients_ = std::exchange(other.coefficients_, nullptr);
    size_ = std::exchange(other.size_, 0);
    return *this;
}

template <typename Fr> Polynomial<Fr> Polynomial<Fr>::share() const
{
    Polynomial p;
    p.backing_memory_ = backing_memory_;
    p.size_ = size_;
    p.coefficients_ = coefficients_;
    return p;
}

template <typename Fr> Fr Polynomial<Fr>::evaluate(const Fr& z, const size_t target_size) const
{
    return polynomial_arithmetic::evaluate(coefficients_, z, target_size);
}

template <typename Fr> Fr Polynomial<Fr>::evaluate(const Fr& z) const
{
    return polynomial_arithmetic::evaluate(coefficients_, z, size_);
}

template <typename Fr> bool Polynomial<Fr>::operator==(Polynomial const& rhs) const
{
    // If either is empty, both must be
    if (is_empty() || rhs.is_empty()) {
        return is_empty() && rhs.is_empty();
    }
    // Size must agree
    if (size() != rhs.size()) {
        return false;
    }
    // Each coefficient must agree
    for (size_t i = 0; i < size(); i++) {
        if (coefficients_[i] != rhs.coefficients_[i]) {
            return false;
        }
    }
    return true;
}

/**
 * @brief sets a block of memory to all zeroes
 * Used to zero out unintialized memory to ensure that, when writing to the polynomial in future,
 * memory requests made to the OS do not return virtual pages (performance optimization).
 * Used, for example, when one polynomial is instantiated from another one with size_>= other.size_.
 *
 * @param opening_proof Opening proof computed by `batch_open`
 * @param commitment_data Describes each polynomial being opened: its commitment, the opening points used and the
 * polynomial evaluations
 */
template <typename Fr> void Polynomial<Fr>::zero_memory_beyond(const size_t start_position)
{
    size_t end = capacity();
    ASSERT(end >= start_position);

    size_t delta = end - start_position;
    if (delta > 0) {
        ASSERT(backing_memory_);
        memset(static_cast<void*>(&coefficients_[start_position]), 0, sizeof(Fr) * delta);
    }
}

/**
 * FFTs
 **/

template <typename Fr>
void Polynomial<Fr>::fft(const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::fft(coefficients_, domain);
}

template <typename Fr>
void Polynomial<Fr>::partial_fft(const EvaluationDomain<Fr>& domain, Fr constant, bool is_coset)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::partial_fft(coefficients_, domain, constant, is_coset);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft(const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft(coefficients_, domain);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft(const EvaluationDomain<Fr>& domain,
                               const EvaluationDomain<Fr>& large_domain,
                               const size_t domain_extension)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    size_t extended_size = domain.size * domain_extension;

    ASSERT(in_place_operation_viable(extended_size));
    zero_memory_beyond(extended_size);

    polynomial_arithmetic::coset_fft(coefficients_, domain, large_domain, domain_extension);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft_with_constant(coefficients_, domain, constant);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft_with_generator_shift(const EvaluationDomain<Fr>& domain, const Fr& constant)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft_with_generator_shift(coefficients_, domain, constant);
}

template <typename Fr>
void Polynomial<Fr>::ifft(const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::ifft(coefficients_, domain);
}

template <typename Fr>
void Polynomial<Fr>::ifft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::ifft_with_constant(coefficients_, domain, constant);
}

template <typename Fr>
void Polynomial<Fr>::coset_ifft(const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_ifft(coefficients_, domain);
}

template <typename Fr>
Fr Polynomial<Fr>::compute_kate_opening_coefficients(const Fr& z)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    return polynomial_arithmetic::compute_kate_opening_coefficients(coefficients_, coefficients_, z, size_);
}

template <typename Fr>
Fr Polynomial<Fr>::compute_barycentric_evaluation(const Fr& z, const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    return polynomial_arithmetic::compute_barycentric_evaluation(coefficients_, domain.size, z, domain);
}

template <typename Fr>
Fr Polynomial<Fr>::evaluate_from_fft(const EvaluationDomain<Fr>& large_domain,
                                     const Fr& z,
                                     const EvaluationDomain<Fr>& small_domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>

{
    return polynomial_arithmetic::evaluate_from_fft(coefficients_, large_domain, z, small_domain);
}

template <typename Fr> Polynomial<Fr> Polynomial<Fr>::shifted() const
{
    ASSERT(size_ > 0);
    ASSERT(coefficients_[0].is_zero());
    ASSERT(coefficients_[size_].is_zero()); // relies on MAXIMUM_COEFFICIENT_SHIFT >= 1
    Polynomial p;
    p.backing_memory_ = backing_memory_;
    p.size_ = size_;
    p.coefficients_ = coefficients_ + 1;
    return p;
}

// TODO(#723): This method is used for the transcript aggregation protocol. For convenience we currently enforce that
// the shift is the same size as the input but this does not need to be the case. Revisit the logic/assertions in this
// method when that issue is addressed.
template <typename Fr> void Polynomial<Fr>::set_to_right_shifted(std::span<Fr> coeffs_in, size_t shift_size)
{
    // Ensure we're not trying to shift self
    ASSERT(coefficients_ != coeffs_in.data());

    auto size_in = coeffs_in.size();
    ASSERT(size_in > 0);

    // Ensure that the last shift_size-many input coefficients are zero to ensure no information is lost in the shift.
    ASSERT(shift_size <= size_in);
    for (size_t i = 0; i < shift_size; ++i) {
        size_t idx = size_in - shift_size - 1;
        ASSERT(coeffs_in[idx].is_zero());
    }

    // Set size of self equal to size of input and allocate memory
    allocate_backing_memory(size_in);

    // Zero out the first shift_size-many coefficients of self
    memset(static_cast<void*>(coefficients_), 0, sizeof(Fr) * shift_size);

    // Copy all but the last shift_size many input coeffs into self at the shift_size-th index.
    std::size_t num_to_copy = size_ - shift_size;
    memcpy(static_cast<void*>(coefficients_ + shift_size),
           static_cast<void const*>(coeffs_in.data()),
           sizeof(Fr) * num_to_copy);
    zero_memory_beyond(size_);
}

template <typename Fr> void Polynomial<Fr>::add_scaled(std::span<const Fr> other, Fr scaling_factor)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    // Calculates number of threads with thread_utils::calculate_num_threads
    size_t num_threads = thread_utils::calculate_num_threads(other_size);
    size_t range_per_thread = other_size / num_threads;
    size_t leftovers = other_size - (range_per_thread * num_threads);
    parallel_for(num_threads, [&](size_t j) {
        size_t offset = j * range_per_thread;
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            coefficients_[i] += scaling_factor * other[i];
        }
    });
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator+=(std::span<const Fr> other)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    size_t num_threads = thread_utils::calculate_num_threads(other_size);
    size_t range_per_thread = other_size / num_threads;
    size_t leftovers = other_size - (range_per_thread * num_threads);
    parallel_for(num_threads, [&](size_t j) {
        size_t offset = j * range_per_thread;
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            coefficients_[i] += other[i];
        }
    });

    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator-=(std::span<const Fr> other)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    size_t num_threads = thread_utils::calculate_num_threads(other_size);
    size_t range_per_thread = other_size / num_threads;
    size_t leftovers = other_size - (range_per_thread * num_threads);
    parallel_for(num_threads, [&](size_t j) {
        size_t offset = j * range_per_thread;
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            coefficients_[i] -= other[i];
        }
    });

    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator*=(const Fr scaling_factor)
{
    ASSERT(in_place_operation_viable());

    size_t num_threads = thread_utils::calculate_num_threads(size_);
    size_t range_per_thread = size_ / num_threads;
    size_t leftovers = size_ - (range_per_thread * num_threads);
    parallel_for(num_threads, [&](size_t j) {
        size_t offset = j * range_per_thread;
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            coefficients_[i] *= scaling_factor;
        }
    });

    return *this;
}

template <typename Fr> Fr Polynomial<Fr>::evaluate_mle(std::span<const Fr> evaluation_points, bool shift) const
{
    const size_t m = evaluation_points.size();

    // To simplify handling of edge cases, we assume that size_ is always a power of 2
    ASSERT(size_ == static_cast<size_t>(1 << m));

    // we do m rounds l = 0,...,m-1.
    // in round l, n_l is the size of the buffer containing the polynomial partially evaluated
    // at u₀,..., u_l.
    // in round 0, this is half the size of n
    size_t n_l = 1 << (m - 1);

    // temporary buffer of half the size of the polynomial
    // TODO(AD): Make this a polynomial with DontZeroMemory::FLAG
    pointer tmp_ptr = _allocate_aligned_memory<Fr>(sizeof(Fr) * n_l);
    auto tmp = tmp_ptr.get();

    Fr* prev = coefficients_;
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

    // Assert that the size of the polynomial being evaluated is a power of 2 greater than (1 << m)
    ASSERT(numeric::is_power_of_two(size_));
    ASSERT(size_ >= static_cast<size_t>(1 << m));
    size_t n = numeric::get_msb(size_);

    // Partial evaluation is done in m rounds l = 0,...,m-1. At the end of round l, the polynomial has been partially
    // evaluated at u_{m-l-1}, ..., u_{m-1} in variables X_{n-l-1}, ..., X_{n-1}. The size of this polynomial is n_l.
    size_t n_l = 1 << (n - 1);

    // Temporary buffer of half the size of the polynomial
    Polynomial<Fr> intermediate(n_l, DontZeroMemory::FLAG);

    // Evaluate variable X_{n-1} at u_{m-1}
    Fr u_l = evaluation_points[m - 1];

    for (size_t i = 0; i < n_l; i++) {
        // Initiate our intermediate results using this polynomial.
        intermediate[i] = at(i) + u_l * (at(i + n_l) - at(i));
    }
    // Evaluate m-1 variables X_{n-l-1}, ..., X_{n-2} at m-1 remaining values u_0,...,u_{m-2})
    for (size_t l = 1; l < m; ++l) {
        n_l = 1 << (n - l - 1);
        u_l = evaluation_points[m - l - 1];
        for (size_t i = 0; i < n_l; ++i) {
            intermediate[i] += u_l * (intermediate[i + n_l] - intermediate[i]);
        }
    }

    // Construct resulting polynomial g(X_0,…,X_{n-m-1})) = p(X_0,…,X_{n-m-1},u_0,...u_{m-1}) from buffer
    Polynomial<Fr> result(n_l, DontZeroMemory::FLAG);
    for (size_t idx = 0; idx < n_l; ++idx) {
        result[idx] = intermediate[idx];
    }

    return result;
}

template class Polynomial<bb::fr>;
template class Polynomial<grumpkin::fr>;

} // namespace bb