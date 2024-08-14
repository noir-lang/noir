#pragma once
#include "barretenberg/common/mem.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/polynomials/shared_shifted_virtual_zeroes_array.hpp"
#include "evaluation_domain.hpp"
#include "polynomial_arithmetic.hpp"
#include <fstream>

namespace bb {

/**
 * @brief Structured polynomial class that represents the coefficients 'a' of a_0 + a_1 x + a_n x^n of
 * a finite field polynomial equation of degree that is at most the size of some zk circuit.
 * Past 'n' it has a virtual size where it conceptually has coefficients all equal to 0.
 * Notably, we allow indexing past 'n' up to our virtual size (checked only in a debug build, however).
 * The polynomial is used to represent the gates of our arithmetized zk programs.
 * Polynomials use the majority of the memory in proving, so caution should be used in making sure
 * unnecessary copies are avoided, both for avoiding unnecessary memory usage and performance
 * due to unnecessary allocations.
 * The polynomial has a maximum degree in the underlying SharedShiftedVirtualZeroesArray, dictated by the circuit size,
 * this is just used for debugging as we represent.
 *
 * @tparam Fr the finite field type.
 */
template <typename Fr> class Polynomial {
  public:
    using FF = Fr;
    enum class DontZeroMemory { FLAG };

    Polynomial(size_t size, size_t virtual_size);
    // Intended just for plonk, where size == virtual_size always
    Polynomial(size_t size)
        : Polynomial(size, size)
    {}
    // Constructor that does not initialize values, use with caution to save time.
    Polynomial(size_t size, size_t virtual_size, DontZeroMemory flag);
    Polynomial(const Polynomial& other);
    Polynomial(const Polynomial& other, size_t target_size);

    Polynomial(Polynomial&& other) noexcept = default;

    Polynomial(std::span<const Fr> coefficients, size_t virtual_size);

    Polynomial(std::span<const Fr> coefficients)
        : Polynomial(coefficients, coefficients.size())
    {}

    // Allow polynomials to be entirely reset/dormant
    Polynomial() = default;

    /**
     * @brief Create the degree-(m-1) polynomial T(X) that interpolates the given evaluations.
     * We have T(xⱼ) = yⱼ for j=1,...,m
     *
     * @param interpolation_points (x₁,…,xₘ)
     * @param evaluations (y₁,…,yₘ)
     */
    Polynomial(std::span<const Fr> interpolation_points, std::span<const Fr> evaluations, size_t virtual_size);

    // move assignment
    Polynomial& operator=(Polynomial&& other) noexcept = default;
    Polynomial& operator=(const Polynomial& other);
    ~Polynomial() = default;

    /**
     * Return a shallow clone of the polynomial. i.e. underlying memory is shared.
     */
    Polynomial share() const;

    void clear() { coefficients_ = SharedShiftedVirtualZeroesArray<Fr>{}; }

    /**
     * @brief Check whether or not a polynomial is identically zero
     *
     */
    bool is_zero() const
    {
        if (is_empty()) {
            ASSERT(false);
            info("Checking is_zero on an empty Polynomial!");
        }
        for (size_t i = 0; i < size(); i++) {
            if (coefficients_.data()[i] != 0) {
                return false;
            }
        }
        return true;
    }

    bool operator==(Polynomial const& rhs) const;

    void set(size_t i, const Fr& value) { coefficients_.set(i, value); };
    Fr get(size_t i) const { return coefficients_.get(i); };

    bool is_empty() const { return coefficients_.size_ == 0; }

    Fr* begin() { return data(); }
    Fr* end() { return data() + size(); }
    const Fr* begin() const { return data(); }
    const Fr* end() const { return data() + size(); }

    /**
     * @brief Returns a Polynomial the left-shift of self.
     *
     * @details If the n coefficients of self are (0, a₁, …, aₙ₋₁),
     * we returns the view of the n-1 coefficients (a₁, …, aₙ₋₁).
     */
    Polynomial shifted() const;

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
     * @return DensePolynomial<Fr> g(X_0,…,X_{n-m-1})) = p(X_0,…,X_{n-m-1},u_0,...u_{m-1})
     */
    Polynomial partial_evaluate_mle(std::span<const Fr> evaluation_points) const;

    Fr compute_barycentric_evaluation(const Fr& z, const EvaluationDomain<Fr>& domain)
        requires polynomial_arithmetic::SupportsFFT<Fr>;
    Fr compute_kate_opening_coefficients(const Fr& z)
        requires polynomial_arithmetic::SupportsFFT<Fr>;

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

    Fr evaluate(const Fr& z, size_t target_size) const;
    Fr evaluate(const Fr& z) const;

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

    std::span<const Fr> as_span() const { return { coefficients_.data(), coefficients_.data() + coefficients_.size_ }; }
    std::span<Fr> as_span() { return { coefficients_.data(), coefficients_.data() + coefficients_.size_ }; }
    std::size_t size() const { return coefficients_.size_; }
    std::size_t virtual_size() const { return coefficients_.virtual_size_; }

    Fr* data() { return coefficients_.data(); }
    const Fr* data() const { return coefficients_.data(); }
    Fr& operator[](size_t i)
    {
        ASSERT(i < size());
        return coefficients_.data()[i];
    }
    const Fr& operator[](size_t i) const
    {
        ASSERT(i < size());
        return coefficients_.data()[i];
    }

    static Polynomial random(size_t size) { return random(size, size); }

    static Polynomial random(size_t size, size_t virtual_size)
    {
        Polynomial p(size, virtual_size, DontZeroMemory::FLAG);
        std::generate_n(p.coefficients_.data(), size, []() { return Fr::random_element(); });
        return p;
    }

  private:
    // allocate a fresh memory pointer for backing memory
    // DOES NOT initialize memory
    void allocate_backing_memory(size_t size, size_t virtual_size);

    // safety check for in place operations
    bool in_place_operation_viable(size_t domain_size = 0) { return (size() >= domain_size); }

    void zero_memory_beyond(size_t start_position);
    // When a polynomial is instantiated from a size alone, the memory allocated corresponds to
    // input size + MAXIMUM_COEFFICIENT_SHIFT to support 'shifted' coefficients efficiently.
    const static size_t MAXIMUM_COEFFICIENT_SHIFT = 1;

    // The underlying memory, with a bespoke (but minimal) shared array struct that fits our needs.
    // Namely, it supports polynomial shifts and 'virtual' zeroes past a size up until a 'virtual' size.
    SharedShiftedVirtualZeroesArray<Fr> coefficients_;
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

} // namespace bb