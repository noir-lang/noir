#include "polynomial.hpp"
#include "polynomial_arithmetic.hpp"
#include <common/assert.hpp>
#include <common/mem.hpp>
#include <common/throw_or_abort.hpp>
#include <cstddef>
#include <sys/stat.h>
#include <fcntl.h>
#include <utility>
#ifndef __wasm__
#include <sys/mman.h>
#endif

namespace barretenberg {

/**
 * Constructors / Destructors
 **/
template <typename Fr>
Polynomial<Fr>::Polynomial(std::string const& filename)
    : mapped_(true)
{
    struct stat st;
    if (stat(filename.c_str(), &st) != 0) {
        throw_or_abort("Filename not found: " + filename);
    }
    size_t len = (size_t)st.st_size;
    size_ = len / sizeof(Fr);
    int fd = open(filename.c_str(), O_RDONLY);
#ifndef __wasm__
    coefficients_ = (Fr*)mmap(0, len, PROT_READ, MAP_PRIVATE, fd, 0);
#else
    coefficients_ = (Fr*)aligned_alloc(32, len);
    ::read(fd, (void*)coefficients_, len);
#endif
    close(fd);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(const size_t size_)
    : coefficients_(nullptr)
    , size_(size_)
    , mapped_(false)
{
    if (capacity() > 0) {
        coefficients_ = (Fr*)(aligned_alloc(32, sizeof(Fr) * capacity()));
    }
    memset(static_cast<void*>(coefficients_), 0, sizeof(Fr) * capacity());
}

template <typename Fr>
Polynomial<Fr>::Polynomial(const Polynomial<Fr>& other, const size_t target_size)
    : size_(std::max(target_size, other.size()))
    , mapped_(false)
{
    coefficients_ = (Fr*)(aligned_alloc(32, sizeof(Fr) * capacity()));

    if (other.coefficients_ != nullptr) {
        memcpy(static_cast<void*>(coefficients_), static_cast<void*>(other.coefficients_), sizeof(Fr) * other.size_);
    }
    zero_memory_beyond(other.size_);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(Polynomial<Fr>&& other) noexcept
    : coefficients_(std::exchange(other.coefficients_, nullptr))
    , size_(std::exchange(other.size_, 0))
    , mapped_(std::exchange(other.mapped_, false))
{}

template <typename Fr>
Polynomial<Fr>::Polynomial(Fr* buf, const size_t size_)
    : coefficients_(buf)
    , size_(size_)
    , mapped_(false)
{}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(const Polynomial<Fr>& other)
{
    if (is_empty()) {
        size_ = other.size();
        coefficients_ = (Fr*)(aligned_alloc(32, sizeof(Fr) * other.capacity()));
    }

    ASSERT(in_place_operation_viable(other.size_));
    zero_memory_beyond(other.size_);

    if (other.coefficients_ != nullptr) {
        memcpy(static_cast<void*>(coefficients_), static_cast<void*>(other.coefficients_), sizeof(Fr) * other.size_);
    }

    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(Polynomial&& other) noexcept
{
    if (&other == this) {
        return *this;
    }
    free();

    // simultaneously set members and clear other
    coefficients_ = std::exchange(other.coefficients_, nullptr);
    size_ = std::exchange(other.size_, 0);
    mapped_ = std::exchange(other.mapped_, false);

    return *this;
}

template <typename Fr> Polynomial<Fr>::~Polynomial()
{
    free();
}

// #######

template <typename Fr> Fr Polynomial<Fr>::evaluate(const Fr& z, const size_t target_size) const
{
    return polynomial_arithmetic::evaluate(coefficients_, z, target_size);
}

template <typename Fr> Fr Polynomial<Fr>::evaluate(const Fr& z) const
{
    return polynomial_arithmetic::evaluate(coefficients_, z, size_);
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
    size_t end = size();
    ASSERT(end >= start_position);

    size_t delta = end - start_position;
    if (delta > 0) {
        ASSERT(coefficients_);
        memset(static_cast<void*>(&coefficients_[start_position]), 0, sizeof(Fr) * delta);
    }
}

template <typename Fr> void Polynomial<Fr>::free()
{
    if (coefficients_ != nullptr) {
#ifndef __wasm__
        if (mapped_) {
            munmap(coefficients_, size_ * sizeof(Fr));
        } else {
            aligned_free(coefficients_);
        }
#else
        aligned_free(coefficients_);
#endif
    }
    coefficients_ = nullptr;
}

/**
 * FFTs
 **/

template <typename Fr> void Polynomial<Fr>::fft(const EvaluationDomain<Fr>& domain)
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::fft(coefficients_, domain);
}

template <typename Fr> void Polynomial<Fr>::partial_fft(const EvaluationDomain<Fr>& domain, Fr constant, bool is_coset)
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::partial_fft(coefficients_, domain, constant, is_coset);
}

template <typename Fr> void Polynomial<Fr>::coset_fft(const EvaluationDomain<Fr>& domain)
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft(coefficients_, domain);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft(const EvaluationDomain<Fr>& domain,
                               const EvaluationDomain<Fr>& large_domain,
                               const size_t domain_extension)
{
    size_t extended_size = domain.size * domain_extension;

    ASSERT(in_place_operation_viable(extended_size));
    zero_memory_beyond(extended_size);

    polynomial_arithmetic::coset_fft(coefficients_, domain, large_domain, domain_extension);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft_with_constant(coefficients_, domain, constant);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft_with_generator_shift(const EvaluationDomain<Fr>& domain, const Fr& constant)
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_fft_with_generator_shift(coefficients_, domain, constant);
}

template <typename Fr> void Polynomial<Fr>::ifft(const EvaluationDomain<Fr>& domain)
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::ifft(coefficients_, domain);
}

template <typename Fr> void Polynomial<Fr>::ifft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::ifft_with_constant(coefficients_, domain, constant);
}

template <typename Fr> void Polynomial<Fr>::coset_ifft(const EvaluationDomain<Fr>& domain)
{
    ASSERT(in_place_operation_viable(domain.size));
    zero_memory_beyond(domain.size);

    polynomial_arithmetic::coset_ifft(coefficients_, domain);
}

template <typename Fr> Fr Polynomial<Fr>::compute_kate_opening_coefficients(const Fr& z)
{
    return polynomial_arithmetic::compute_kate_opening_coefficients(coefficients_, coefficients_, z, size_);
}

template <typename Fr>
Fr Polynomial<Fr>::compute_barycentric_evaluation(const Fr& z, const EvaluationDomain<Fr>& domain)
{
    return polynomial_arithmetic::compute_barycentric_evaluation(coefficients_, domain.size, z, domain);
}

template <typename Fr>
Fr Polynomial<Fr>::evaluate_from_fft(const EvaluationDomain<Fr>& large_domain,
                                     const Fr& z,
                                     const EvaluationDomain<Fr>& small_domain)
{
    return polynomial_arithmetic::evaluate_from_fft(coefficients_, large_domain, z, small_domain);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(std::span<const Fr> interpolation_points, std::span<const Fr> evaluations)
    : Polynomial(interpolation_points.size())
{
    ASSERT(size_ > 0);

    polynomial_arithmetic::compute_efficient_interpolation(
        evaluations.data(), coefficients_, interpolation_points.data(), size_);
}

template <typename Fr> void Polynomial<Fr>::add_scaled(std::span<const Fr> other, Fr scaling_factor)
{
    const size_t other_size = other.size();
    ASSERT(in_place_operation_viable(other_size));

    /** TODO parallelize using some kind of generic evaluation domain
     *  we really only need to know the thread size, but we don't need all the FFT roots
     */
    for (size_t i = 0; i < other_size; ++i) {
        coefficients_[i] += scaling_factor * other[i];
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
        coefficients_[i] += other[i];
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
        coefficients_[i] -= other[i];
    }

    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator*=(const Fr scaling_facor)
{
    ASSERT(in_place_operation_viable());

    for (size_t i = 0; i < size_; ++i) {
        coefficients_[i] *= scaling_facor;
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
    Fr* tmp = static_cast<Fr*>(aligned_alloc(sizeof(Fr), sizeof(Fr) * n_l));

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
    // free the temporary buffer
    aligned_free(tmp);
    return result;
}

template class Polynomial<barretenberg::fr>;
template class Polynomial<grumpkin::fr>;

} // namespace barretenberg