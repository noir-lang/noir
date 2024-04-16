#pragma once
#include "barretenberg/common/assert.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include <span>

namespace bb {

/**
 * @brief A view of a univariate, also used to truncate univariates.
 *
 * @details For optimization purposes, it makes sense to define univariates with large lengths and then reuse only some
 * of the data in those univariates. We do that by taking a view of those elements and then, as needed, using this to
 * populate new containers.
 */
template <class Fr, size_t view_domain_end, size_t view_domain_start> class UnivariateView;

/**
 * @brief A univariate polynomial represented by its values on {domain_start, domain_start + 1,..., domain_end - 1}. For
 * memory efficiency purposes, we store the evaluations in an array starting from 0 and make the mapping to the right
 * domain under the hood.
 */
template <class Fr, size_t domain_end, size_t domain_start = 0> class Univariate {
  public:
    static constexpr size_t LENGTH = domain_end - domain_start;
    using View = UnivariateView<Fr, domain_end, domain_start>;

    using value_type = Fr; // used to get the type of the elements consistently with std::array

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/714) Try out std::valarray?
    std::array<Fr, LENGTH> evaluations;

    Univariate() = default;

    explicit Univariate(std::array<Fr, LENGTH> evaluations)
        : evaluations(evaluations)
    {}
    ~Univariate() = default;
    Univariate(const Univariate& other) = default;
    Univariate(Univariate&& other) noexcept = default;
    Univariate& operator=(const Univariate& other) = default;
    Univariate& operator=(Univariate&& other) noexcept = default;
    // Construct constant Univariate from scalar which represents the value that all the points in the domain evaluate
    // to
    explicit Univariate(Fr value)
        : evaluations{}
    {
        for (size_t i = 0; i < LENGTH; ++i) {
            evaluations[i] = value;
        }
    }
    // Construct Univariate from UnivariateView
    explicit Univariate(UnivariateView<Fr, domain_end, domain_start> in)
        : evaluations{}
    {
        for (size_t i = 0; i < in.evaluations.size(); ++i) {
            evaluations[i] = in.evaluations[i];
        }
    }

    Fr& value_at(size_t i) { return evaluations[i - domain_start]; };
    const Fr& value_at(size_t i) const { return evaluations[i - domain_start]; };
    size_t size() { return evaluations.size(); };

    // Write the Univariate evaluations to a buffer
    [[nodiscard]] std::vector<uint8_t> to_buffer() const { return ::to_buffer(evaluations); }

    // Static method for creating a Univariate from a buffer
    // IMPROVEMENT: Could be made to identically match equivalent methods in e.g. field.hpp. Currently bypasses
    // unnecessary ::from_buffer call
    static Univariate serialize_from_buffer(uint8_t const* buffer)
    {
        Univariate result;
        std::read(buffer, result.evaluations);
        return result;
    }

    static Univariate get_random()
    {
        auto output = Univariate<Fr, domain_end, domain_start>();
        for (size_t i = 0; i != LENGTH; ++i) {
            output.value_at(i) = Fr::random_element();
        }
        return output;
    };

    static Univariate zero()
    {
        auto output = Univariate<Fr, domain_end, domain_start>();
        for (size_t i = 0; i != LENGTH; ++i) {
            output.value_at(i) = Fr::zero();
        }
        return output;
    }

    static Univariate random_element() { return get_random(); };

    // Operations between Univariate and other Univariate
    bool operator==(const Univariate& other) const = default;

    Univariate& operator+=(const Univariate& other)
    {
        for (size_t i = 0; i < LENGTH; ++i) {
            evaluations[i] += other.evaluations[i];
        }
        return *this;
    }
    Univariate& operator-=(const Univariate& other)
    {
        for (size_t i = 0; i < LENGTH; ++i) {
            evaluations[i] -= other.evaluations[i];
        }
        return *this;
    }
    Univariate& operator*=(const Univariate& other)
    {
        for (size_t i = 0; i < LENGTH; ++i) {
            evaluations[i] *= other.evaluations[i];
        }
        return *this;
    }
    Univariate& self_sqr()
    {
        for (size_t i = 0; i < LENGTH; ++i) {
            evaluations[i].self_sqr();
        }
        return *this;
    }
    Univariate operator+(const Univariate& other) const
    {
        Univariate res(*this);
        res += other;
        return res;
    }

    Univariate operator-(const Univariate& other) const
    {
        Univariate res(*this);
        res -= other;
        return res;
    }
    Univariate operator-() const
    {
        Univariate res(*this);
        for (auto& eval : res.evaluations) {
            eval = -eval;
        }
        return res;
    }

    Univariate operator*(const Univariate& other) const
    {
        Univariate res(*this);
        res *= other;
        return res;
    }

    Univariate sqr() const
    {
        Univariate res(*this);
        res.self_sqr();
        return res;
    }

    // Operations between Univariate and scalar
    Univariate& operator+=(const Fr& scalar)
    {
        for (auto& eval : evaluations) {
            eval += scalar;
        }
        return *this;
    }

    Univariate& operator-=(const Fr& scalar)
    {
        for (auto& eval : evaluations) {
            eval -= scalar;
        }
        return *this;
    }
    Univariate& operator*=(const Fr& scalar)
    {
        for (auto& eval : evaluations) {
            eval *= scalar;
        }
        return *this;
    }

    Univariate operator+(const Fr& scalar) const
    {
        Univariate res(*this);
        res += scalar;
        return res;
    }

    Univariate operator-(const Fr& scalar) const
    {
        Univariate res(*this);
        res -= scalar;
        return res;
    }

    Univariate operator*(const Fr& scalar) const
    {
        Univariate res(*this);
        res *= scalar;
        return res;
    }

    // Operations between Univariate and UnivariateView
    Univariate& operator+=(const UnivariateView<Fr, domain_end, domain_start>& view)
    {
        for (size_t i = 0; i < LENGTH; ++i) {
            evaluations[i] += view.evaluations[i];
        }
        return *this;
    }

    Univariate& operator-=(const UnivariateView<Fr, domain_end, domain_start>& view)
    {
        for (size_t i = 0; i < LENGTH; ++i) {
            evaluations[i] -= view.evaluations[i];
        }
        return *this;
    }

    Univariate& operator*=(const UnivariateView<Fr, domain_end, domain_start>& view)
    {
        for (size_t i = 0; i < LENGTH; ++i) {
            evaluations[i] *= view.evaluations[i];
        }
        return *this;
    }

    Univariate operator+(const UnivariateView<Fr, domain_end, domain_start>& view) const
    {
        Univariate res(*this);
        res += view;
        return res;
    }

    Univariate operator-(const UnivariateView<Fr, domain_end, domain_start>& view) const
    {
        Univariate res(*this);
        res -= view;
        return res;
    }

    Univariate operator*(const UnivariateView<Fr, domain_end, domain_start>& view) const
    {
        Univariate res(*this);
        res *= view;
        return res;
    }

    // Output is immediately parsable as a list of integers by Python.
    friend std::ostream& operator<<(std::ostream& os, const Univariate& u)
    {
        os << "[";
        os << u.evaluations[0] << "," << std::endl;
        for (size_t i = 1; i < u.evaluations.size(); i++) {
            os << " " << u.evaluations[i];
            if (i + 1 < u.evaluations.size()) {
                os << "," << std::endl;
            } else {
                os << "]";
            };
        }
        return os;
    }

    /**
     * @brief Given a univariate f represented by {f(domain_start), ..., f(domain_end - 1)}, compute the evaluations
     * {f(domain_end),..., f(extended_domain_end -1)} and return the Univariate represented by  {f(domain_start),...,
     * f(extended_domain_end -1)}
     *
     * @details Write v_i = f(x_i) on a the domain {x_{domain_start}, ..., x_{domain_end-1}}. To efficiently compute the
     * needed values of f, we use the barycentric formula
     *      - f(x) = B(x) Σ_{i=domain_start}^{domain_end-1} v_i / (d_i*(x-x_i))
     * where
     *      - B(x) = Π_{i=domain_start}^{domain_end-1} (x-x_i)
     *      - d_i  = Π_{j ∈ {domain_start, ..., domain_end-1}, j≠i} (x_i-x_j) for i ∈ {domain_start, ..., domain_end-1}
     *
     * When the domain size is two, extending f = v0(1-X) + v1X to a new value involves just one addition and a
     * subtraction: setting Δ = v1-v0, the values of f(X) are f(0)=v0, f(1)= v0 + Δ, v2 = f(1) + Δ, v3 = f(2) + Δ...
     *
     */
    template <size_t EXTENDED_DOMAIN_END> Univariate<Fr, EXTENDED_DOMAIN_END> extend_to() const
    {
        const size_t EXTENDED_LENGTH = EXTENDED_DOMAIN_END - domain_start;
        using Data = BarycentricData<Fr, LENGTH, EXTENDED_LENGTH>;
        static_assert(EXTENDED_LENGTH >= LENGTH);

        Univariate<Fr, EXTENDED_LENGTH> result;

        std::copy(evaluations.begin(), evaluations.end(), result.evaluations.begin());

        static constexpr Fr inverse_two = Fr(2).invert();
        if constexpr (LENGTH == 2) {
            Fr delta = value_at(1) - value_at(0);
            static_assert(EXTENDED_LENGTH != 0);
            for (size_t idx = domain_end - 1; idx < EXTENDED_DOMAIN_END - 1; idx++) {
                result.value_at(idx + 1) = result.value_at(idx) + delta;
            }
            return result;
        } else if constexpr (LENGTH == 3) {
            // Based off https://hackmd.io/@aztec-network/SyR45cmOq?type=view
            // The technique used here is the same as the length == 3 case below.
            Fr a = (value_at(2) + value_at(0)) * inverse_two - value_at(1);
            Fr b = value_at(1) - a - value_at(0);
            Fr a2 = a + a;
            Fr a_mul = a2;
            for (size_t i = 0; i < domain_end - 2; i++) {
                a_mul += a2;
            }
            Fr extra = a_mul + a + b;
            for (size_t idx = domain_end - 1; idx < EXTENDED_DOMAIN_END - 1; idx++) {
                result.value_at(idx + 1) = result.value_at(idx) + extra;
                extra += a2;
            }
            return result;
        } else if constexpr (LENGTH == 4) {
            static constexpr Fr inverse_six = Fr(6).invert(); // computed at compile time for efficiency

            // To compute a barycentric extension, we can compute the coefficients of the univariate.
            // We have the evaluation of the polynomial at the domain (which is assumed to be 0, 1, 2, 3).
            // Therefore, we have the 4 linear equations from plugging into f(x) = ax^3 + bx^2 + cx + d:
            //          a*0 + b*0 + c*0 + d = f(0)
            //          a*1 + b*1 + c*1 + d = f(1)
            //          a*2^3 + b*2^2 + c*2 + d = f(2)
            //          a*3^3 + b*3^2 + c*3 + d = f(3)
            // These equations can be rewritten as a matrix equation M * [a, b, c, d] = [f(0), f(1), f(2), f(3)], where
            // M is:
            //          0,  0,  0,  1
            //          1,  1,  1,  1
            //          2^3, 2^2, 2,  1
            //          3^3, 3^2, 3,  1
            // We can invert this matrix in order to compute a, b, c, d:
            //      -1/6,	1/2,	-1/2,	1/6
            //      1,	    -5/2,	2,	    -1/2
            //      -11/6,	3,	    -3/2,	1/3
            //      1,	    0,	    0,	    0
            // To compute these values, we can multiply everything by 6 and multiply by inverse_six at the end for each
            // coefficient The resulting computation here does 18 field adds, 6 subtracts, 3 muls to compute a, b, c,
            // and d.
            Fr zero_times_3 = value_at(0) + value_at(0) + value_at(0);
            Fr zero_times_6 = zero_times_3 + zero_times_3;
            Fr zero_times_12 = zero_times_6 + zero_times_6;
            Fr one_times_3 = value_at(1) + value_at(1) + value_at(1);
            Fr one_times_6 = one_times_3 + one_times_3;
            Fr two_times_3 = value_at(2) + value_at(2) + value_at(2);
            Fr three_times_2 = value_at(3) + value_at(3);
            Fr three_times_3 = three_times_2 + value_at(3);

            Fr one_minus_two_times_3 = one_times_3 - two_times_3;
            Fr one_minus_two_times_6 = one_minus_two_times_3 + one_minus_two_times_3;
            Fr one_minus_two_times_12 = one_minus_two_times_6 + one_minus_two_times_6;
            Fr a = (one_minus_two_times_3 + value_at(3) - value_at(0)) * inverse_six; // compute a in 1 muls and 4 adds
            Fr b = (zero_times_6 - one_minus_two_times_12 - one_times_3 - three_times_3) * inverse_six;
            Fr c = (value_at(0) - zero_times_12 + one_minus_two_times_12 + one_times_6 + two_times_3 + three_times_2) *
                   inverse_six;

            // Then, outside of the a, b, c, d computation, we need to do some extra precomputation
            // This work is 3 field muls, 8 adds
            Fr a_plus_b = a + b;
            Fr a_plus_b_times_2 = a_plus_b + a_plus_b;
            size_t start_idx_sqr = (domain_end - 1) * (domain_end - 1);
            size_t idx_sqr_three = start_idx_sqr + start_idx_sqr + start_idx_sqr;
            Fr idx_sqr_three_times_a = Fr(idx_sqr_three) * a;
            Fr x_a_term = Fr(6 * (domain_end - 1)) * a;
            Fr three_a = a + a + a;
            Fr six_a = three_a + three_a;

            Fr three_a_plus_two_b = a_plus_b_times_2 + a;
            Fr linear_term = Fr(domain_end - 1) * three_a_plus_two_b + (a_plus_b + c);
            // For each new evaluation, we do only 6 field additions and 0 muls.
            for (size_t idx = domain_end - 1; idx < EXTENDED_DOMAIN_END - 1; idx++) {
                result.value_at(idx + 1) = result.value_at(idx) + idx_sqr_three_times_a + linear_term;

                idx_sqr_three_times_a += x_a_term + three_a;
                x_a_term += six_a;

                linear_term += three_a_plus_two_b;
            }
            return result;
        } else {
            for (size_t k = domain_end; k != EXTENDED_DOMAIN_END; ++k) {
                result.value_at(k) = 0;
                // compute each term v_j / (d_j*(x-x_j)) of the sum
                for (size_t j = domain_start; j != domain_end; ++j) {
                    Fr term = value_at(j);
                    term *= Data::precomputed_denominator_inverses[LENGTH * k + j];
                    result.value_at(k) += term;
                }
                // scale the sum by the the value of of B(x)
                result.value_at(k) *= Data::full_numerator_values[k];
            }
            return result;
        }
    }

    /**
     * @brief Evaluate a univariate at a point u not known at compile time
     * and assumed not to be in the domain (else we divide by zero).
     * @param f
     * @return Fr
     */
    Fr evaluate(const Fr& u)
    {
        using Data = BarycentricData<Fr, domain_end, LENGTH, domain_start>;
        Fr full_numerator_value = 1;
        for (size_t i = domain_start; i != domain_end; ++i) {
            full_numerator_value *= u - i;
        }

        // build set of domain size-many denominator inverses 1/(d_i*(x_k - x_j)). will multiply against each of
        // these (rather than to divide by something) for each barycentric evaluation
        std::array<Fr, LENGTH> denominator_inverses;
        for (size_t i = 0; i != LENGTH; ++i) {
            Fr inv = Data::lagrange_denominators[i];
            inv *= u - Data::big_domain[i]; // warning: need to avoid zero here
            inv = Fr(1) / inv;
            denominator_inverses[i] = inv;
        }

        Fr result = 0;
        // compute each term v_j / (d_j*(x-x_j)) of the sum
        for (size_t i = domain_start; i != domain_end; ++i) {
            Fr term = value_at(i);
            term *= denominator_inverses[i - domain_start];
            result += term;
        }
        // scale the sum by the the value of of B(x)
        result *= full_numerator_value;
        return result;
    };

    // Begin iterators
    auto begin() { return evaluations.begin(); }
    auto begin() const { return evaluations.begin(); }
    // End iterators
    auto end() { return evaluations.end(); }
    auto end() const { return evaluations.end(); }
};

template <typename B, class Fr, size_t domain_end, size_t domain_start = 0>
inline void read(B& it, Univariate<Fr, domain_end, domain_start>& univariate)
{
    using serialize::read;
    read(it, univariate.evaluations);
}

template <typename B, class Fr, size_t domain_end, size_t domain_start = 0>
inline void write(B& it, Univariate<Fr, domain_end, domain_start> const& univariate)
{
    using serialize::write;
    write(it, univariate.evaluations);
}

template <class Fr, size_t domain_end, size_t domain_start = 0> class UnivariateView {
  public:
    static constexpr size_t LENGTH = domain_end - domain_start;
    std::span<const Fr, LENGTH> evaluations;

    UnivariateView() = default;

    const Fr& value_at(size_t i) const { return evaluations[i]; };

    template <size_t full_domain_end, size_t full_domain_start = 0>
    explicit UnivariateView(const Univariate<Fr, full_domain_end, full_domain_start>& univariate_in)
        : evaluations(std::span<const Fr>(univariate_in.evaluations.data(), LENGTH)){};

    Univariate<Fr, domain_end, domain_start> operator+(const UnivariateView& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res += other;
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator-(const UnivariateView& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res -= other;
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator-() const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        for (auto& eval : res.evaluations) {
            eval = -eval;
        }
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator*(const UnivariateView& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res *= other;
        return res;
    }
    Univariate<Fr, domain_end, domain_start> sqr() const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res = res.sqr();
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator*(const Univariate<Fr, domain_end, domain_start>& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res *= other;
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator+(const Univariate<Fr, domain_end, domain_start>& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res += other;
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator+(const Fr& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res += other;
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator-(const Fr& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res -= other;
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator*(const Fr& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res *= other;
        return res;
    }

    Univariate<Fr, domain_end, domain_start> operator-(const Univariate<Fr, domain_end, domain_start>& other) const
    {
        Univariate<Fr, domain_end, domain_start> res(*this);
        res -= other;
        return res;
    }

    // Output is immediately parsable as a list of integers by Python.
    friend std::ostream& operator<<(std::ostream& os, const UnivariateView& u)
    {
        os << "[";
        os << u.evaluations[0] << "," << std::endl;
        for (size_t i = 1; i < u.evaluations.size(); i++) {
            os << " " << u.evaluations[i];
            if (i + 1 < u.evaluations.size()) {
                os << "," << std::endl;
            } else {
                os << "]";
            };
        }
        return os;
    }
};

/**
 * @brief Create a sub-array of `elements` at the indices given in the template pack `Is`, converting them to the new
 * type T.
 *
 * @tparam T type to convert to
 * @tparam U type to convert from
 * @tparam N number (deduced by `elements`)
 * @tparam Is list of indices we want in the returned array. When the second argument is called with
 * `std::make_index_sequence<N>`, these will be `0, 1, ..., N-1`.
 * @param elements array to convert from
 * @return std::array<T, sizeof...(Is)> result array s.t. result[i] = T(elements[Is[i]]). By default, Is[i] = i when
 * called with `std::make_index_sequence<N>`.
 */
template <typename T, typename U, std::size_t N, std::size_t... Is>
std::array<T, sizeof...(Is)> array_to_array_aux(const std::array<U, N>& elements, std::index_sequence<Is...>)
{
    return { { T{ elements[Is] }... } };
};

/**
 * @brief Given an std::array<U,N>, returns an std::array<T,N>, by calling the (explicit) constructor T(U).
 *
 * @details https://stackoverflow.com/a/32175958
 * The main use case is to convert an array of `Univariate` into `UnivariateView`. The main use case would be to let
 * Sumcheck decide the required degree of the relation evaluation, rather than hardcoding it inside the relation. The
 * `_aux` version could also be used to create an array of only the polynomials required by the relation, and it could
 * help us implement the optimization where we extend each edge only up to the maximum degree that is required over all
 * relations (for example, `L_LAST` only needs degree 3).
 *
 * @tparam T Output type
 * @tparam U Input type (deduced from `elements`)
 * @tparam N Common array size (deduced from `elements`)
 * @param elements array to be converted
 * @return std::array<T, N> result s.t. result[i] = T(elements[i])
 */
template <typename T, typename U, std::size_t N> std::array<T, N> array_to_array(const std::array<U, N>& elements)
{
    // Calls the aux method that uses the index sequence to unpack all values in `elements`
    return array_to_array_aux<T, U, N>(elements, std::make_index_sequence<N>());
};

} // namespace bb

namespace std {
template <typename T, size_t N> struct tuple_size<bb::Univariate<T, N>> : std::integral_constant<std::size_t, N> {};

} // namespace std