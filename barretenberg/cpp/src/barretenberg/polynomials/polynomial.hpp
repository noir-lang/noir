#pragma once
#include "barretenberg/common/mem.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "evaluation_domain.hpp"
#include "polynomial_arithmetic.hpp"
#include <fstream>

namespace bb {
enum class DontZeroMemory { FLAG };

template <typename Fr> class Polynomial {
  public:
    /**
     * Implements requirements of `std::ranges::contiguous_range` and `std::ranges::sized_range`
     */
    using value_type = Fr;
    using difference_type = std::ptrdiff_t;
    using reference = value_type&;
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
    using pointer = std::shared_ptr<value_type[]>;
    using const_pointer = pointer;
    using iterator = Fr*;
    using const_iterator = Fr const*;
    using FF = Fr;

    Polynomial(size_t initial_size);
    // Constructor that does not initialize values, use with caution to save time.
    Polynomial(size_t initial_size, DontZeroMemory flag);
    Polynomial(const Polynomial& other);
    Polynomial(const Polynomial& other, size_t target_size);

    Polynomial(Polynomial&& other) noexcept;

    // Create a polynomial from the given fields.
    Polynomial(std::span<const Fr> coefficients);

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

    // move assignment
    Polynomial& operator=(Polynomial&& other) noexcept;
    Polynomial& operator=(std::span<const Fr> coefficients) noexcept;
    Polynomial& operator=(const Polynomial& other);
    ~Polynomial() = default;

    /**
     * Return a shallow clone of the polynomial. i.e. underlying memory is shared.
     */
    Polynomial share() const;

    std::array<uint8_t, 32> hash() const { return sha256::sha256(byte_span()); }

    void clear()
    {
        // to keep the invariant that backing_memory_ can handle capacity() we do NOT reset backing_memory_
        // backing_memory_.reset();
        coefficients_ = nullptr;
        size_ = 0;
    }

    bool operator==(Polynomial const& rhs) const;

    // Const and non const versions of coefficient accessors
    Fr const& operator[](const size_t i) const { return coefficients_[i]; }

    Fr& operator[](const size_t i) { return coefficients_[i]; }

    Fr const& at(const size_t i) const
    {
        ASSERT(i < capacity());
        return coefficients_[i];
    };

    Fr& at(const size_t i)
    {
        ASSERT(i < capacity());
        return coefficients_[i];
    };

    Fr evaluate(const Fr& z, size_t target_size) const;
    Fr evaluate(const Fr& z) const;

    Fr compute_barycentric_evaluation(const Fr& z, const EvaluationDomain<Fr>& domain)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    Fr evaluate_from_fft(const EvaluationDomain<Fr>& large_domain,
                         const Fr& z,
                         const EvaluationDomain<Fr>& small_domain)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void fft(const EvaluationDomain<Fr>& domain)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void partial_fft(const EvaluationDomain<Fr>& domain, Fr constant = 1, bool is_coset = false)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void coset_fft(const EvaluationDomain<Fr>& domain)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void coset_fft(const EvaluationDomain<Fr>& domain,
                   const EvaluationDomain<Fr>& large_domain,
                   size_t domain_extension)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void coset_fft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void coset_fft_with_generator_shift(const EvaluationDomain<Fr>& domain, const Fr& constant)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void ifft(const EvaluationDomain<Fr>& domain)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void ifft_with_constant(const EvaluationDomain<Fr>& domain, const Fr& constant)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    void coset_ifft(const EvaluationDomain<Fr>& domain)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    Fr compute_kate_opening_coefficients(const Fr& z)
        requires polynomial_arithmetic::SupportsFFT<Fr>;

    bool is_empty() const { return size_ == 0; }

    /**
     * @brief Returns an std::span of the left-shift of self.
     *
     * @details If the n coefficients of self are (0, a₁, …, aₙ₋₁),
     * we returns the view of the n-1 coefficients (a₁, …, aₙ₋₁).
     */
    Polynomial shifted() const;

    /**
     * @brief Set self to the right shift of input coefficients
     * @details Set the size of self to match the input then set coefficients equal to right shift of input. Note: The
     * shifted result is constructed with its first shift-many coefficients equal to zero, so we assert that the last
     * shift-size many input coefficients are equal to zero to ensure that the relationship f(X) = f_{shift}(X)/X^m
     * holds. This is analagous to asserting the first coefficient is 0 in our left-shift-by-one method.
     *
     * @param coeffs_in
     * @param shift_size
     */
    void set_to_right_shifted(std::span<Fr> coeffs_in, size_t shift_size = 1);

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
    Polynomial& operator*=(Fr scaling_factor);

    /**
     * @brief evaluates p(X) = ∑ᵢ aᵢ⋅Xⁱ considered as multi-linear extension p(X₀,…,Xₘ₋₁) = ∑ᵢ aᵢ⋅Lᵢ(X₀,…,Xₘ₋₁)
     * at u = (u₀,…,uₘ₋₁)
     *
     * @details this function allocates a temporary buffer of size n/2
     *
     * @param evaluation_points an MLE evaluation point u = (u₀,…,uₘ₋₁)
     * @param shift evaluates p'(X₀,…,Xₘ₋₁) = 1⋅L₀(X₀,…,Xₘ₋₁) + ∑ᵢ˲₁ aᵢ₋₁⋅Lᵢ(X₀,…,Xₘ₋₁) if true
     * @return Fr p(u₀,…,uₘ₋₁)
     */
    Fr evaluate_mle(std::span<const Fr> evaluation_points, bool shift = false) const;

    /**
     * @brief Partially evaluates in the last k variables a polynomial interpreted as a multilinear extension.
     *
     * @details Partially evaluates p(X) = (a_0, ..., a_{2^n-1}) considered as multilinear extension p(X_0,…,X_{n-1}) =
     * \sum_i a_i*L_i(X_0,…,X_{n-1}) at u = (u_0,…,u_{m-1}), m < n, in the last m variables X_n-m,…,X_{n-1}. The result
     * is a multilinear polynomial in n-m variables g(X_0,…,X_{n-m-1})) = p(X_0,…,X_{n-m-1},u_0,...u_{m-1}).
     *
     * @note Intuitively, partially evaluating in one variable collapses the hypercube in one dimension, halving the
     * number of coefficients needed to represent the result. To partially evaluate starting with the first variable (as
     * is done in evaluate_mle), the vector of coefficents is halved by combining adjacent rows in a pairwise
     * fashion (similar to what is done in Sumcheck via "edges"). To evaluate starting from the last variable, we
     * instead bisect the whole vector and combine the two halves. I.e. rather than coefficents being combined with
     * their immediate neighbor, they are combined with the coefficient that lives n/2 indices away.
     *
     * @param evaluation_points an MLE partial evaluation point u = (u_0,…,u_{m-1})
     * @return Polynomial<Fr> g(X_0,…,X_{n-m-1})) = p(X_0,…,X_{n-m-1},u_0,...u_{m-1})
     */
    Polynomial<Fr> partial_evaluate_mle(std::span<const Fr> evaluation_points) const;

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

    iterator begin() { return coefficients_; }
    iterator end() { return coefficients_ + size_; }
    pointer data() { return backing_memory_; }

    std::span<uint8_t> byte_span() const
    {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
        return { reinterpret_cast<uint8_t*>(coefficients_), size_ * sizeof(Fr) };
    }

    const_iterator begin() const { return coefficients_; }
    const_iterator end() const { return coefficients_ + size_; }
    const_pointer data() const { return backing_memory_; }

    std::size_t size() const { return size_; }
    std::size_t capacity() const { return size_ + MAXIMUM_COEFFICIENT_SHIFT; }

  private:
    // allocate a fresh memory pointer for backing memory
    // DOES NOT initialize memory
    void allocate_backing_memory(size_t n_elements);

    // safety check for in place operations
    bool in_place_operation_viable(size_t domain_size = 0) { return (size() >= domain_size); }

    void zero_memory_beyond(size_t start_position);
    // When a polynomial is instantiated from a size alone, the memory allocated corresponds to
    // input size + MAXIMUM_COEFFICIENT_SHIFT to support 'shifted' coefficients efficiently.
    const static size_t MAXIMUM_COEFFICIENT_SHIFT = 1;

    // The memory
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
    std::shared_ptr<Fr[]> backing_memory_;
    // A pointer into backing_memory_ to support std::span-like functionality. This allows for coefficient subsets
    // and shifts.
    Fr* coefficients_ = nullptr;
    // The size_ effectively represents the 'usable' length of the coefficients array but may be less than the true
    // 'capacity' of the array. It is not explicitly tied to the degree and is not changed by any operations on the
    // polynomial.
    size_t size_ = 0;
};

template <typename Fr> inline std::ostream& operator<<(std::ostream& os, Polynomial<Fr> const& p)
{
    if (p.size() == 0) {
        return os << "[]";
    }
    if (p.size() == 1) {
        return os << "[ data " << p[0] << "]";
    }
    return os << "[ data\n"
              << "  " << p[0] << ",\n"
              << "  " << p[1] << ",\n"
              << "  ... ,\n"
              << "  " << p[p.size() - 2] << ",\n"
              << "  " << p[p.size() - 1] << ",\n"
              << "]";
}

using polynomial = Polynomial<bb::fr>;

} // namespace bb

/**
 * The static_assert below ensure that that our Polynomial class correctly models an `std::ranges::contiguous_range`,
 * and other requirements that allow us to convert a `Polynomial<Fr>` to a `std::span<const Fr>`.
 *
 * This also means we can now iterate over the elements in the vector using a `for(auto ...)` loop, and use various std
 * algorithms.
 *
 * static_assert(std::ranges::contiguous_range<bb::polynomial>);
 * static_assert(std::ranges::sized_range<bb::polynomial>);
 * static_assert(std::convertible_to<bb::polynomial, std::span<const bb::fr>>);
 * static_assert(std::convertible_to<bb::polynomial&, std::span<bb::fr>>);
 * // cannot convert a const polynomial to a non-const span
 * static_assert(!std::convertible_to<const bb::polynomial&, std::span<bb::fr>>);
 * static_assert(std::convertible_to<const bb::polynomial&, std::span<const bb::fr>>);
 */
