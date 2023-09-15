#pragma once
#include "barretenberg/common/assert.hpp"
#include "barretenberg/common/serialize.hpp"
#include <array>
#include <span>

namespace barretenberg {

/**
 * @brief A view of a univariate, also used to truncate univariates.
 *
 * @details For optimization purposes, it makes sense to define univariates with large lengths and then reuse only some
 * of the data in those univariates. We do that by taking a view of those elements and then, as needed, using this to
 * populate new containers.
 */
template <class Fr, size_t view_length> class UnivariateView;

/**
 * @brief A univariate polynomial represented by its values on {0,1,..., _length-1}
 */
template <class Fr, size_t _length> class Univariate {
  public:
    static constexpr size_t LENGTH = _length;

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/714) Try out std::valarray?
    std::array<Fr, _length> evaluations;

    Univariate() = default;

    explicit Univariate(std::array<Fr, _length> evaluations)
        : evaluations(evaluations)
    {}
    ~Univariate() = default;
    Univariate(const Univariate& other) = default;
    Univariate(Univariate&& other) noexcept = default;
    Univariate& operator=(const Univariate& other) = default;
    Univariate& operator=(Univariate&& other) noexcept = default;
    // Construct Univariate from scalar
    explicit Univariate(Fr value)
        : evaluations{}
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] = value;
        }
    }
    // Construct Univariate from UnivariateView
    explicit Univariate(UnivariateView<Fr, _length> in)
        : evaluations{}
    {
        for (size_t i = 0; i < in.evaluations.size(); ++i) {
            evaluations[i] = in.evaluations[i];
        }
    }

    Fr& value_at(size_t i) { return evaluations[i]; };
    const Fr& value_at(size_t i) const { return evaluations[i]; };

    // Write the Univariate evaluations to a buffer
    std::vector<uint8_t> to_buffer() const { return ::to_buffer(evaluations); }

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
        auto output = Univariate<Fr, _length>();
        for (size_t i = 0; i != _length; ++i) {
            output.value_at(i) = Fr::random_element();
        }
        return output;
    };

    // Operations between Univariate and other Univariate
    bool operator==(const Univariate& other) const = default;

    Univariate& operator+=(const Univariate& other)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] += other.evaluations[i];
        }
        return *this;
    }
    Univariate& operator-=(const Univariate& other)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] -= other.evaluations[i];
        }
        return *this;
    }
    Univariate& operator*=(const Univariate& other)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] *= other.evaluations[i];
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
    Univariate& operator+=(const UnivariateView<Fr, _length>& view)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] += view.evaluations[i];
        }
        return *this;
    }

    Univariate& operator-=(const UnivariateView<Fr, _length>& view)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] -= view.evaluations[i];
        }
        return *this;
    }

    Univariate& operator*=(const UnivariateView<Fr, _length>& view)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] *= view.evaluations[i];
        }
        return *this;
    }

    Univariate operator+(const UnivariateView<Fr, _length>& view) const
    {
        Univariate res(*this);
        res += view;
        return res;
    }

    Univariate operator-(const UnivariateView<Fr, _length>& view) const
    {
        Univariate res(*this);
        res -= view;
        return res;
    }

    Univariate operator*(const UnivariateView<Fr, _length>& view) const
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
};

template <typename B, class Fr, size_t _length> inline void read(B& it, Univariate<Fr, _length>& univariate)
{
    using serialize::read;
    read(it, univariate.evaluations);
}

template <typename B, class Fr, size_t _length> inline void write(B& it, Univariate<Fr, _length> const& univariate)
{
    using serialize::write;
    write(it, univariate.evaluations);
}

template <class Fr, size_t view_length> class UnivariateView {
  public:
    std::span<const Fr, view_length> evaluations;

    UnivariateView() = default;

    const Fr& value_at(size_t i) const { return evaluations[i]; };

    template <size_t full_length>
    explicit UnivariateView(const Univariate<Fr, full_length>& univariate_in)
        : evaluations(std::span<const Fr>(univariate_in.evaluations.data(), view_length)){};

    Univariate<Fr, view_length> operator+(const UnivariateView& other) const
    {
        Univariate<Fr, view_length> res(*this);
        res += other;
        return res;
    }

    Univariate<Fr, view_length> operator-(const UnivariateView& other) const
    {
        Univariate<Fr, view_length> res(*this);
        res -= other;
        return res;
    }

    Univariate<Fr, view_length> operator-() const
    {
        Univariate res(*this);
        for (auto& eval : res.evaluations) {
            eval = -eval;
        }
        return res;
    }

    Univariate<Fr, view_length> operator*(const UnivariateView& other) const
    {
        Univariate<Fr, view_length> res(*this);
        res *= other;
        return res;
    }

    Univariate<Fr, view_length> operator*(const Univariate<Fr, view_length>& other) const
    {
        Univariate<Fr, view_length> res(*this);
        res *= other;
        return res;
    }

    Univariate<Fr, view_length> operator+(const Univariate<Fr, view_length>& other) const
    {
        Univariate<Fr, view_length> res(*this);
        res += other;
        return res;
    }

    Univariate<Fr, view_length> operator+(const Fr& other) const
    {
        Univariate<Fr, view_length> res(*this);
        res += other;
        return res;
    }

    Univariate<Fr, view_length> operator-(const Fr& other) const
    {
        Univariate<Fr, view_length> res(*this);
        res -= other;
        return res;
    }

    Univariate<Fr, view_length> operator*(const Fr& other) const
    {
        Univariate<Fr, view_length> res(*this);
        res *= other;
        return res;
    }

    Univariate<Fr, view_length> operator-(const Univariate<Fr, view_length>& other) const
    {
        Univariate<Fr, view_length> res(*this);
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

} // namespace barretenberg
