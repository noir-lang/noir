#pragma once
#include "evaluation_domain.hpp"
#include <cstddef>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <common/mem.hpp>
#include <common/timer.hpp>
#include <fstream>
#include <concepts>
#include <span>
#include "polynomial_arithmetic.hpp"

namespace barretenberg {
template <typename Fr> class Polynomial {
  public:
    // Creates a read only polynomial using mmap.
    Polynomial(std::string const& filename);

    Polynomial(const size_t initial_size);
    Polynomial(const Polynomial& other, const size_t target_size = 0);

    Polynomial(Polynomial&& other) noexcept;

    // Takes ownership of given buffer.
    Polynomial(Fr* buf, const size_t initial_size);

    // Allow polynomials to be entirely reset/dormant
    Polynomial() = default;

    /**
     * @brief Create the degree-(m-1) polynomial T(X) that interpolates the given evaluations.
     * We have T(xⱼ) = yⱼ for j=1,...,m
     *
     * @param interpolation_points (x₁,…,xₘ)
     * @param evaluations (y₁,…,yₘ)
     */
    Polynomial(std::span<const Fr> interpolation_points, std::span<const Fr> evaluations);

    Polynomial& operator=(Polynomial&& other) noexcept;
    Polynomial& operator=(const Polynomial& other);
    ~Polynomial();

    void clear()
    {
        free();

        coefficients_ = 0;
        size_ = 0;
        mapped_ = false;
    }

    bool operator==(Polynomial const& rhs) const
    {
        // If either is empty, both must be
        if (is_empty() || rhs.is_empty()) {
            return is_empty() && rhs.is_empty();
        }
        // Otherwise, check that the coefficients match on the minimum of the two sizes and that the higher coefficients
        // of the larger poly (if one exists) are identically zero.
        else {
            size_t min_size = std::min(size(), rhs.size());
            for (size_t i = 0; i < min_size; i++) {
                if (coefficients_[i] != rhs.coefficients_[i])
                    return false;
            }
            for (size_t i = min_size; i < size(); i++) {
                if (coefficients_[i] != 0)
                    return false;
            }
            for (size_t i = min_size; i < rhs.size(); i++) {
                if (rhs.coefficients_[i] != 0)
                    return false;
            }

            return true;
        }
    }

    // IMPROVEMENT: deprecate in favor of 'data()' and ensure const correctness
    Fr* get_coefficients() const { return coefficients_; };
    Fr* get_coefficients() { return coefficients_; };

    // Const and non const versions of coefficient accessors
    Fr const& operator[](const size_t i) const
    {
        ASSERT(i < size_);
        return coefficients_[i];
    }

    Fr& operator[](const size_t i)
    {
        ASSERT(i < size_);
        return coefficients_[i];
    }

    Fr const& at(const size_t i) const
    {
        ASSERT(i < size_);
        return coefficients_[i];
    };

    Fr& at(const size_t i)
    {
        ASSERT(i < size_);
        return coefficients_[i];
    };

    Fr evaluate(const Fr& z, const size_t target_size) const;
    Fr evaluate(const Fr& z) const;
    Fr compute_barycentric_evaluation(const Fr& z, const EvaluationDomain<Fr>& domain);

    Fr evaluate_from_fft(const EvaluationDomain<Fr>& large_domain,
                         const Fr& z,
                         const EvaluationDomain<Fr>& small_domain);

    void fft(const EvaluationDomain<Fr>& domain);
    void partial_fft(const EvaluationDomain<Fr>& domain, Fr constant = 1, bool is_coset = false);
    void coset_fft(const EvaluationDomain<Fr>& domain);
    void coset_fft(const EvaluationDomain<Fr>& domain,
                   const EvaluationDomain<Fr>& large_domain,
                   const size_t domain_extension);

    void coset_fft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& costant);
    void coset_fft_with_generator_shift(const EvaluationDomain<Fr>& domain, const Fr& constant);

    void ifft(const EvaluationDomain<Fr>& domain);
    void ifft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant);
    void coset_ifft(const EvaluationDomain<Fr>& domain);

    Fr compute_kate_opening_coefficients(const Fr& z);

    bool is_empty() const { return (coefficients_ == nullptr) || (size_ == 0); }

    // safety check for in place operations
    bool in_place_operation_viable(size_t domain_size = 0)
    {
        return !is_empty() && !mapped() && (size() >= domain_size);
    }

    /**
     * @brief Returns an std::span of the left-shift of self.
     *
     * @details If the n coefficients of self are (0, a₁, …, aₙ₋₁),
     * we returns the view of the n-1 coefficients (a₁, …, aₙ₋₁).
     */
    std::span<Fr> shifted() const
    {
        ASSERT(size_ > 0);
        ASSERT(coefficients_[0].is_zero());
        ASSERT(coefficients_[size_].is_zero()); // relies on DEFAULT_CAPACITY_INCREASE >= 1
        return std::span{ coefficients_ + 1, size_ };
    }

    /**
     * @brief adds the polynomial q(X) 'other', multiplied by a scaling factor.
     *
     * @param other q(X)
     * @param scaling_factor scaling factor by which all coefficients of q(X) are multiplied
     */
    void add_scaled(std::span<const Fr> other, Fr scaling_factor);

    /**
     * @brief adds the polynomial q(X) 'other'.
     *
     * @param other q(X)
     */
    Polynomial& operator+=(std::span<const Fr> other);

    /**
     * @brief subtracts the polynomial q(X) 'other'.
     *
     * @param other q(X)
     */
    Polynomial& operator-=(std::span<const Fr> other);

    /**
     * @brief sets this = p(X) to s⋅p(X)
     *
     * @param scaling_factor s
     */
    Polynomial& operator*=(const Fr scaling_facor);

    /**
     * @brief evaluates p(X) = ∑ᵢ aᵢ⋅Xⁱ considered as multi-linear extension p(X₁,…,Xₘ) = ∑ᵢ aᵢ⋅Lᵢ(X₁,…,Xₘ)
     * at u = (u₁,…,uₘ)
     *
     * @details this function allocates a temporary buffer of size n/2
     *
     * @param evaluation_points an MLE evaluation point u = (u₁,…,uₘ)
     * @param shift evaluates p'(X₁,…,Xₘ) = 1⋅L₀(X₁,…,Xₘ) + ∑ᵢ˲₁ aᵢ₋₁⋅Lᵢ(X₁,…,Xₘ) if true
     * @return Fr p(u₁,…,uₘ)
     */
    Fr evaluate_mle(std::span<const Fr> evaluation_points, bool shift = false) const;

    /**
     * @brief Divides p(X) by (X-r₁)⋯(X−rₘ) in-place.
     * Assumes that p(rⱼ)=0 for all j
     *
     * @details we specialize the method when only a single root is given.
     * if one of the roots is 0, then we first factor all other roots.
     * dividing by X requires only a left shift of all coefficient.
     *
     * @param roots list of roots (r₁,…,rₘ)
     */
    void factor_roots(std::span<const Fr> roots) { polynomial_arithmetic::factor_roots(std::span{ *this }, roots); };
    void factor_roots(const Fr& root) { polynomial_arithmetic::factor_roots(std::span{ *this }, root); };

    /**
     * Implements requirements of `std::ranges::contiguous_range` and `std::ranges::sized_range`
     */
    using value_type = Fr;
    using difference_type = std::ptrdiff_t;
    using reference = value_type&;
    using pointer = value_type*;
    using const_pointer = value_type const*;
    using iterator = pointer;
    using const_iterator = const_pointer;

    iterator begin() { return coefficients_; }
    iterator end() { return coefficients_ + size_; }
    pointer data() { return coefficients_; }

    const_iterator begin() const { return coefficients_; }
    const_iterator end() const { return coefficients_ + size_; }
    const_pointer data() const { return coefficients_; }

    std::size_t size() const { return size_; }
    std::size_t capacity() const { return size_ + DEFAULT_CAPACITY_INCREASE; }
    bool mapped() const { return mapped_; }

  private:
    void free();
    void zero_memory_beyond(const size_t start_position);
    // When a polynomial is instantiated from a size alone, the memory allocated corresponds to
    // input size + DEFAULT_CAPACITY_INCREASE. A DEFAULT_CAPACITY_INCREASE of >= 1 is required to ensure
    // that polynomials can be 'shifted' via a span of the 1st to size+1th coefficients.
    const static size_t DEFAULT_CAPACITY_INCREASE = 1;

  public:
    Fr* coefficients_ = nullptr;
    // The size_ effectively represents the 'usable' length of the coefficients array but may be less than the true
    // 'capacity' of the array. It is not explicitly tied to the degree and is not changed by any operations on the
    // polynomial.
    size_t size_ = 0;
    bool mapped_ = false;
};

template <typename Fr> inline std::ostream& operator<<(std::ostream& os, Polynomial<Fr> const& p)
{
    return os << "[ " << p[0] << ", ... ]";
}

// N.B. grumpkin polynomials don't support fast fourier transforms using roots of unity!
// TODO: use template junk to disable fft methods if Fr::SUPPORTS_FFTS == false
// extern template class Polynomial<grumpkin::fr>;
extern template class Polynomial<barretenberg::fr>;
extern template class Polynomial<grumpkin::fr>;

using polynomial = Polynomial<barretenberg::fr>;

} // namespace barretenberg

/**
 * The static_assert below ensure that that our Polynomial class correctly models an `std::ranges::contiguous_range`,
 * and other requirements that allow us to convert a `Polynomial<Fr>` to a `std::span<const Fr>`.
 *
 * This also means we can now iterate over the elements in the vector using a `for(auto ...)` loop, and use various std
 * algorithms.
 *
 * static_assert(std::ranges::contiguous_range<barretenberg::polynomial>);
 * static_assert(std::ranges::sized_range<barretenberg::polynomial>);
 * static_assert(std::convertible_to<barretenberg::polynomial, std::span<const barretenberg::fr>>);
 * static_assert(std::convertible_to<barretenberg::polynomial&, std::span<barretenberg::fr>>);
 * // cannot convert a const polynomial to a non-const span
 * static_assert(!std::convertible_to<const barretenberg::polynomial&, std::span<barretenberg::fr>>);
 * static_assert(std::convertible_to<const barretenberg::polynomial&, std::span<const barretenberg::fr>>);
 */
