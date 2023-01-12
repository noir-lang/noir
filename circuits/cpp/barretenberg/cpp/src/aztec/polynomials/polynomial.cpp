#include "polynomial.hpp"
#include "polynomial_arithmetic.hpp"
#include <common/assert.hpp>
#include <common/mem.hpp>
#include <common/throw_or_abort.hpp>
#include <sys/stat.h>
#include <fcntl.h>
#ifndef __wasm__
#include <sys/mman.h>
#endif

namespace {
size_t clamp(size_t target, size_t step)
{
    size_t res = (target / step) * step;
    if (res < target)
        res += step;
    return res;
}
} // namespace

namespace barretenberg {

/**
 * Constructors / Destructors
 **/
template <typename Fr>
Polynomial<Fr>::Polynomial(std::string const& filename)
    : mapped_(true)
    , page_size_(DEFAULT_SIZE_HINT)
    , allocated_pages_(0)
{
    struct stat st;
    if (stat(filename.c_str(), &st) != 0) {
        throw_or_abort("Filename not found: " + filename);
    }
    size_t len = (size_t)st.st_size;
    size_ = len / sizeof(Fr);
    max_size_ = size_;
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
Polynomial<Fr>::Polynomial(const size_t initial_size_, const size_t initial_max_size_hint)
    : mapped_(false)
    , coefficients_(nullptr)
    , initial_size_(initial_size_)
    , size_(initial_size_)
    , page_size_(DEFAULT_SIZE_HINT)
    , max_size_(0)
    , allocated_pages_(0)
{
    ASSERT(page_size_ != 0);
    size_t target_max_size = std::max(initial_size_, initial_max_size_hint + DEFAULT_PAGE_SPILL);
    if (target_max_size > 0) {

        max_size_ = target_max_size;
        coefficients_ = (Fr*)(aligned_alloc(32, sizeof(Fr) * target_max_size));
    }
    memset(static_cast<void*>(coefficients_), 0, sizeof(Fr) * max_size_);
}

template <typename Fr>
Polynomial<Fr>::Polynomial(const Polynomial<Fr>& other, const size_t target_max_size)
    : mapped_(false)
    , initial_size_(other.initial_size_)
    , size_(other.size_)
    , page_size_(other.page_size_)
    , max_size_(std::max(clamp(target_max_size, page_size_ + DEFAULT_PAGE_SPILL), other.max_size_))
    , allocated_pages_(max_size_ / page_size_)
{
    ASSERT(page_size_ != 0);
    ASSERT(max_size_ >= size_);

    coefficients_ = (Fr*)(aligned_alloc(32, sizeof(Fr) * max_size_));

    if (other.coefficients_ != nullptr) {
        memcpy(static_cast<void*>(coefficients_), static_cast<void*>(other.coefficients_), sizeof(Fr) * size_);
    }
    zero_memory(max_size_);
}
template <typename Fr>
Polynomial<Fr>::Polynomial(Polynomial<Fr>&& other) noexcept
    : mapped_(other.mapped_)
    , coefficients_(other.coefficients_)
    , initial_size_(other.initial_size_)
    , size_(other.size_)
    , page_size_(other.page_size_)
    , max_size_(other.max_size_)
    , allocated_pages_(other.allocated_pages_)
{
    ASSERT(page_size_ != 0);
    other.coefficients_ = 0;

    // Clear other, so we can detect use on free after poly's are put in cache
    other.clear();
}

template <typename Fr>
Polynomial<Fr>::Polynomial(Fr* buf, const size_t initial_size_)
    : mapped_(false)
    , coefficients_(buf)
    , initial_size_(initial_size_)
    , size_(initial_size_)
    , page_size_(DEFAULT_SIZE_HINT)
    , max_size_(initial_size_)
    , allocated_pages_(0)
{}

template <typename Fr>
Polynomial<Fr>::Polynomial()
    : mapped_(false)
    , coefficients_(0)
    , initial_size_(0)
    , size_(0)
    , page_size_(DEFAULT_SIZE_HINT)
    , max_size_(0)
    , allocated_pages_(0)
{}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(const Polynomial<Fr>& other)
{
    ASSERT(page_size_ != 0);
    mapped_ = false;
    initial_size_ = other.initial_size_;

    // Set page size first so that if we do copy from other we allocate according to
    // other's page size not ours.
    page_size_ = other.page_size_;

    if (other.max_size_ > max_size_) {
        // Bump memory and set max_size, before we set size otherwise we will copy an
        // inappropriate amount of data.
        bump_memory(other.max_size_);
    } else {
        max_size_ = other.max_size_;
        allocated_pages_ = other.allocated_pages_;
    }

    size_ = other.size_;

    ASSERT(max_size_ >= size_);

    if (other.coefficients_ != 0) {
        ASSERT(coefficients_);
        memcpy(static_cast<void*>(coefficients_), static_cast<void*>(other.coefficients_), sizeof(Fr) * size_);
    }

    zero_memory(max_size_);

    return *this;
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator=(Polynomial&& other) noexcept
{
    if (&other == this) {
        return *this;
    }
    free();

    mapped_ = other.mapped_;
    coefficients_ = other.coefficients_;
    page_size_ = other.page_size_;
    max_size_ = other.max_size_;
    allocated_pages_ = other.allocated_pages_;
    initial_size_ = other.initial_size_;
    size_ = other.size_;
    ASSERT(page_size_ != 0);

    other.coefficients_ = nullptr;
    // Clear other, so we can detect use on free after poly's are put in cache
    other.clear();
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
 * Called when re-sizing the container to ensure that, when writing to the polynomial in future,
 * memory requests made to the OS do not return virtual pages (performance optimisation)
 *
 * @param opening_proof Opening proof computed by `batch_open`
 * @param commitment_data Describes each polynomial being opened: its commitment, the opening points used and the
 * polynomial evaluations
 */
template <typename Fr> void Polynomial<Fr>::zero_memory(const size_t zero_size)
{
    ASSERT(zero_size >= size_);

    if (zero_size > size_) {

        size_t delta = zero_size - size_;
        if (delta > 0 && coefficients_) {
            ASSERT(coefficients_);
            memset(static_cast<void*>(&coefficients_[size_]), 0, sizeof(Fr) * delta);
        }
    }
}

template <typename Fr> void Polynomial<Fr>::bump_memory(const size_t new_size_hint)
{
    ASSERT(!mapped_);
    size_t amount = (new_size_hint / page_size_) * page_size_;

    while (amount < new_size_hint) {
        amount += page_size_;
    }

    Fr* new_memory = (Fr*)(aligned_alloc(32, sizeof(Fr) * amount));
    if (coefficients_ != 0) {
        ASSERT(amount >= size_);

        memcpy(new_memory, coefficients_, sizeof(Fr) * size_);
        free();
    }
    coefficients_ = new_memory;
    allocated_pages_ = amount / page_size_;
    max_size_ = amount;
}

template <typename Fr> void Polynomial<Fr>::add_coefficient_internal(const Fr& coefficient)
{
    ASSERT(!mapped_);
    if (size_ + 1 > max_size_) {
        bump_memory((allocated_pages_ + 1) * page_size_);
    }
    Fr::__copy(coefficient, coefficients_[size_]);
    ++size_;
}

template <typename Fr> void Polynomial<Fr>::add_lagrange_base_coefficient(const Fr& coefficient)
{
    add_coefficient_internal(coefficient);
}

template <typename Fr> void Polynomial<Fr>::add_coefficient(const Fr& coefficient)
{
    add_coefficient_internal(coefficient);
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

template <typename Fr> void Polynomial<Fr>::reserve(const size_t amount)
{
    ASSERT(!mapped_);
    if (amount > max_size_) {
        bump_memory(amount);
        memset(static_cast<void*>(&coefficients_[size_]), 0, sizeof(Fr) * (amount - max_size_));
    }
}

template <typename Fr> void Polynomial<Fr>::resize(const size_t amount)
{
    ASSERT(!mapped_);

    if (amount > max_size_) {
        bump_memory(amount);
    }

    if (coefficients_ != 0 && amount > size_) {

        ASSERT(amount > size_);

        Fr* back = &coefficients_[size_];
        memset(static_cast<void*>(back), 0, sizeof(Fr) * (amount - size_));
    }

    size_ = amount;
}

// does not zero out memory
template <typename Fr> void Polynomial<Fr>::resize_unsafe(const size_t amount)
{
    ASSERT(!mapped_);

    if (amount > max_size_) {
        bump_memory(amount);
    }

    size_ = amount;
}

/**
 * FFTs
 **/

template <typename Fr> void Polynomial<Fr>::fft(const EvaluationDomain<Fr>& domain)
{
    ASSERT(!empty());

    if (domain.size > max_size_) {
        bump_memory(domain.size);
    }

    // (ZERO OUT MEMORY!)
    // TODO: wait, do we still need this?
    // memset(static_cast<void*>(back), 0, sizeof(Fr) * (amount - size));
    polynomial_arithmetic::fft(coefficients_, domain);
    size_ = domain.size;
}

template <typename Fr> void Polynomial<Fr>::partial_fft(const EvaluationDomain<Fr>& domain, Fr constant, bool is_coset)
{
    if (domain.size > max_size_) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::partial_fft(coefficients_, domain, constant, is_coset);
    size_ = domain.size;
}

template <typename Fr> void Polynomial<Fr>::coset_fft(const EvaluationDomain<Fr>& domain)
{
    ASSERT(!empty());
    if (domain.size > max_size_) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::coset_fft(coefficients_, domain);
    size_ = domain.size;
}

template <typename Fr>
void Polynomial<Fr>::coset_fft(const EvaluationDomain<Fr>& domain,
                               const EvaluationDomain<Fr>& large_domain,
                               const size_t domain_extension)
{
    ASSERT(!empty());

    if ((domain.size * domain_extension) > max_size_) {
        bump_memory(domain.size * domain_extension);
    }

    polynomial_arithmetic::coset_fft(coefficients_, domain, large_domain, domain_extension);
    size_ = (domain.size * domain_extension);
}

template <typename Fr>
void Polynomial<Fr>::coset_fft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
{
    ASSERT(!empty());

    if (domain.size > max_size_) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::coset_fft_with_constant(coefficients_, domain, constant);
    size_ = domain.size;
}

template <typename Fr>
void Polynomial<Fr>::coset_fft_with_generator_shift(const EvaluationDomain<Fr>& domain, const Fr& constant)
{
    ASSERT(!empty());

    if (domain.size > max_size_) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::coset_fft_with_generator_shift(coefficients_, domain, constant);
    size_ = domain.size;
}

template <typename Fr> void Polynomial<Fr>::ifft(const EvaluationDomain<Fr>& domain)
{
    ASSERT(!empty());

    if (domain.size > max_size_) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::ifft(coefficients_, domain);
    size_ = domain.size;
}

template <typename Fr> void Polynomial<Fr>::ifft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
{
    ASSERT(!empty());

    if (domain.size > max_size_) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::ifft_with_constant(coefficients_, domain, constant);
    size_ = domain.size;
}

template <typename Fr> void Polynomial<Fr>::coset_ifft(const EvaluationDomain<Fr>& domain)
{
    ASSERT(!empty());

    if (domain.size > max_size_) {
        bump_memory(domain.size);
    }

    polynomial_arithmetic::coset_ifft(coefficients_, domain);
    size_ = domain.size;
}

// void Polynomial<Fr>::coset_ifft_with_constant(const EvaluationDomain<Fr> &domain, const Fr &constant)
// {
//     if (domain.size > max_size)
//     {
//         bump_memory(domain.size);
//     }

//     polynomial_arithmetic::coset_ifft_with_constant(coefficients, domain, constant);
// }

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
    : Polynomial(interpolation_points.size(), interpolation_points.size())
{
    ASSERT(size_ > 0);

    polynomial_arithmetic::compute_efficient_interpolation(
        evaluations.data(), coefficients_, interpolation_points.data(), size_);
}

template <typename Fr> void Polynomial<Fr>::add_scaled(std::span<const Fr> other, Fr scaling_factor)
{
    ASSERT(!mapped_);
    const size_t other_size = other.size();
    if (other_size > max_size_) {
        std::cout << "bumping memory! hey this shouldn't really happen in prod!" << std::endl;
        bump_memory(other_size);
    }
    size_ = std::max(size_, other_size);

    /** TODO parallelize using some kind of generic evaluation domain
     *  we really only need to know the thread size, but we don't need all the FFT roots
     */

    for (size_t i = 0; i < other_size; ++i) {
        coefficients_[i] += scaling_factor * other[i];
    }
}

template <typename Fr> Polynomial<Fr>& Polynomial<Fr>::operator+=(std::span<const Fr> other)
{
    ASSERT(!mapped_);
    const size_t other_size = other.size();
    if (other_size > max_size_) {
        std::cout << "bumping memory! hey this shouldn't really happen in prod!" << std::endl;
        bump_memory(other_size);
    }
    size_ = std::max(size_, other_size);

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
    ASSERT(!mapped_);
    const size_t other_size = other.size();
    if (other_size > max_size_) {
        std::cout << "bumping memory! hey this shouldn't really happen in prod!" << std::endl;
        bump_memory(other_size);
    }
    size_ = std::max(size_, other_size);

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
    ASSERT(!mapped_);

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