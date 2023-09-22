#include "polynomial.hpp"
#include "barretenberg/common/assert.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "polynomial_arithmetic.hpp"
#include <cstddef>
#include <fcntl.h>
#include <list>
#include <memory>
#include <mutex>
#include <sys/stat.h>
#include <unordered_map>
#include <utility>

namespace barretenberg {
/**
 * Constructors / Destructors
 **/
template <typename Fr>
Polynomial<Fr>::Polynomial(const size_t size_)
    : coefficients_(nullptr)
    , size_(size_)
{
    if (capacity() > 0) {
        coefficients_ = allocate_aligned_memory(sizeof(Fr) * capacity());
    }
    memset(static_cast<void*>(coefficients_.get()), 0, sizeof(Fr) * capacity());
}

template <typename Fr>
Polynomial<Fr>::Polynomial(const Polynomial<Fr>& other)
    : Polynomial<Fr>(other, other.size())
{}

template <typename Fr>
Polynomial<Fr>::Polynomial(const Polynomial<Fr>& other, const size_t target_size)
    : size_(std::max(target_size, other.size()))
{
    // info("Polynomial EXPENSIVE Copy ctor size ", size_);
    coefficients_ = allocate_aligned_memory(sizeof(Fr) * capacity());

    if (other.coefficients_ != nullptr) {
        memcpy(static_cast<void*>(coefficients_.get()),
               static_cast<void*>(other.coefficients_.get()),
               sizeof(Fr) * other.size_);
    }
    zero_memory_beyond(other.size_);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(Polynomial<Fr>&& other) noexcept
    : coefficients_(std::exchange(other.coefficients_, nullptr))
    , size_(std::exchange(other.size_, 0))
{
    // info("Move ctor Polynomial took ownership of ", coefficients_, " size ", size_);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(std::span<const Fr> coefficients)
    : size_(coefficients.size())
{
    coefficients_ = allocate_aligned_memory(sizeof(Fr) * capacity());
    // info("Polynomial span ctor new buf at ", coefficients_, " size ", size_);
    memcpy(static_cast<void*>(coefficients_.get()),
           static_cast<void const*>(coefficients.data()),
           sizeof(Fr) * coefficients.size());
    zero_memory_beyond(size_);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(std::span<const Fr> interpolation_points, std::span<const Fr> evaluations)
    : Polynomial(interpolation_points.size())
{
    ASSERT(size_ > 0);

    // info("Polynomial INTERPOLATION ctor.");

    polynomial_arithmetic::compute_efficient_interpolation(
        evaluations.data(), coefficients_.get(), interpolation_points.data(), size_);
}

template <typename Fr> Polynomial<Fr>::~Polynomial() {}

// Assignments

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(const Polynomial<Fr>& other)
{
    // info("Polynomial EXPENSIVE copy assignment.");
    size_ = other.size_;

    coefficients_ = allocate_aligned_memory(sizeof(Fr) * capacity());

    if (other.coefficients_ != nullptr) {
        memcpy(static_cast<void*>(coefficients_.get()),
               static_cast<void*>(other.coefficients_.get()),
               sizeof(Fr) * other.size_);
    }
    zero_memory_beyond(size_);
    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(Polynomial&& other) noexcept
{
    if (&other == this) {
        return *this;
    }

    // info("Polynomial move assignment.");
    // simultaneously set members and clear other
    coefficients_ = std::exchange(other.coefficients_, nullptr);
    size_ = std::exchange(other.size_, 0);

    return *this;
}

// #######

template <typename Fr> Fr Polynomial<Fr>::evaluate(const Fr& z, const size_t target_size) const
{
    return polynomial_arithmetic::evaluate(coefficients_.get(), z, target_size);
}

template <typename Fr> Fr Polynomial<Fr>::evaluate(const Fr& z) const
{
    return polynomial_arithmetic::evaluate(coefficients_.get(), z, size_);
}

/**
 * @brief sets a block of memory to all zeroes
 * Used to zero out unintialized memory to ensure that, when writing to the polynomial in future,
 * memory requests made to the OS do not return virtual pages (performance optimisation).
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
        ASSERT(coefficients_);
        memset(static_cast<void*>(&coefficients_.get()[start_position]), 0, sizeof(Fr) * delta);
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

    polynomial_arithmetic::fft(coefficients_.get(), domain);
}

template <typename Fr>
void Polynomial<Fr>::partial_fft(const EvaluationDomain<Fr>& domain, Fr constant, bool is_coset)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::partial_fft(coefficients_.get(), domain, constant, is_coset);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft(const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft(coefficients_.get(), domain);
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

    polynomial_arithmetic::coset_fft(coefficients_.get(), domain, large_domain, domain_extension);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft_with_constant(coefficients_.get(), domain, constant);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft_with_generator_shift(const EvaluationDomain<Fr>& domain, const Fr& constant)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft_with_generator_shift(coefficients_.get(), domain, constant);
}

template <typename Fr>
void Polynomial<Fr>::ifft(const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::ifft(coefficients_.get(), domain);
}

template <typename Fr>
void Polynomial<Fr>::ifft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::ifft_with_constant(coefficients_.get(), domain, constant);
}

template <typename Fr>
void Polynomial<Fr>::coset_ifft(const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_ifft(coefficients_.get(), domain);
}

template <typename Fr>
Fr Polynomial<Fr>::compute_kate_opening_coefficients(const Fr& z)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    return polynomial_arithmetic::compute_kate_opening_coefficients(coefficients_.get(), coefficients_.get(), z, size_);
}

template <typename Fr>
Fr Polynomial<Fr>::compute_barycentric_evaluation(const Fr& z, const EvaluationDomain<Fr>& domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>
{
    return polynomial_arithmetic::compute_barycentric_evaluation(coefficients_.get(), domain.size, z, domain);
}

template <typename Fr>
Fr Polynomial<Fr>::evaluate_from_fft(const EvaluationDomain<Fr>& large_domain,
                                     const Fr& z,
                                     const EvaluationDomain<Fr>& small_domain)
    requires polynomial_arithmetic::SupportsFFT<Fr>

{
    return polynomial_arithmetic::evaluate_from_fft(coefficients_.get(), large_domain, z, small_domain);
}

// TODO(#723): This method is used for the transcript aggregation protocol. For convenience we currently enforce that
// the shift is the same size as the input but this does not need to be the case. Revisit the logic/assertions in this
// method when that issue is addressed.
template <typename Fr> void Polynomial<Fr>::set_to_right_shifted(std::span<Fr> coeffs_in, size_t shift_size)
{
    // Ensure we're not trying to shift self
    ASSERT(coefficients_.get() != coeffs_in.data());

    auto size_in = coeffs_in.size();
    ASSERT(size_in > 0);

    // Ensure that the last shift_size-many input coefficients are zero to ensure no information is lost in the shift.
    ASSERT(shift_size <= size_in);
    for (size_t i = 0; i < shift_size; ++i) {
        size_t idx = size_in - shift_size - 1;
        ASSERT(coeffs_in[idx].is_zero());
    }

    // Set size of self equal to size of input and allocate memory
    size_ = size_in;
    coefficients_ = allocate_aligned_memory(sizeof(Fr) * capacity());

    // Zero out the first shift_size-many coefficients of self
    memset(static_cast<void*>(coefficients_.get()), 0, sizeof(Fr) * shift_size);

    // Copy all but the last shift_size many input coeffs into self at the shift_size-th index.
    std::size_t num_to_copy = size_ - shift_size;
    memcpy(static_cast<void*>(coefficients_.get() + shift_size),
           static_cast<void const*>(coeffs_in.data()),
           sizeof(Fr) * num_to_copy);
    zero_memory_beyond(size_);
}

template <typename Fr> void Polynomial<Fr>::add_scaled(std::span<const Fr> other, Fr scaling_factor)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    /** TODO parallelize using some kind of generic evaluation domain
     *  we really only need to know the thread size, but we don't need all the FFT roots
     */
    for (size_t i = 0; i < other_size; ++i) {
        coefficients_.get()[i] += scaling_factor * other[i];
    }
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator+=(std::span<const Fr> other)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    /** TODO parallelize using some kind of generic evaluation domain
     *  we really only need to know the thread size, but we don't need all the FFT roots
     */
    for (size_t i = 0; i < other_size; ++i) {
        coefficients_.get()[i] += other[i];
    }

    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator-=(std::span<const Fr> other)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    /** TODO parallelize using some kind of generic evaluation domain
     *  we really only need to know the thread size, but we don't need all the FFT roots
     */
    for (size_t i = 0; i < other_size; ++i) {
        coefficients_.get()[i] -= other[i];
    }

    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator*=(const Fr scaling_facor)
{
    ASSERT(in_place_operation_viable());

    for (size_t i = 0; i < size_; ++i) {
        coefficients_.get()[i] *= scaling_facor;
    }
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
    pointer tmp_ptr = allocate_aligned_memory(sizeof(Fr) * n_l);
    auto tmp = tmp_ptr.get();

    Fr* prev = coefficients_.get();
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

template <typename Fr> typename Polynomial<Fr>::pointer Polynomial<Fr>::allocate_aligned_memory(const size_t size) const
{
    return std::static_pointer_cast<Fr[]>(get_mem_slab(size));
}

template class Polynomial<barretenberg::fr>;
template class Polynomial<grumpkin::fr>;

} // namespace barretenberg
