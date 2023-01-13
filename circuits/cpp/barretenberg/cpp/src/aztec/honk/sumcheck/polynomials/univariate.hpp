#pragma once
#include <array>
#include <cstdint>
#include <span>
#include <ostream>
#include <common/serialize.hpp>
#include <common/assert.hpp>

namespace honk::sumcheck {

template <class Fr, size_t view_length> class UnivariateView;

// TODO(Cody): This violates Rule of Five
template <class Fr, size_t _length> class Univariate {
  public:
    static constexpr size_t LENGTH = _length;

    std::array<Fr, _length> evaluations;

    Univariate() = default;

    explicit Univariate(std::array<Fr, _length> evaluations)
        : evaluations(evaluations)
    {}
    Univariate(const Univariate& other)
        : evaluations(other.evaluations)
    {}
    Univariate(Univariate&& other) noexcept
        : evaluations(std::move(other.evaluations))
    {}

    // Construct Univariate from UnivariateView
    explicit Univariate(UnivariateView<Fr, _length> in)
        : evaluations{}
    {
        for (size_t i = 0; i < in.evaluations.size(); ++i) {
            evaluations[i] = in.evaluations[i];
        }
    }

    Fr& value_at(size_t i) { return evaluations[i]; };

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

    // Operations between Univariate and other Univariate
    Univariate operator=(const Univariate& other)
    {
        evaluations = other.evaluations;
        return *this;
    }

    Univariate operator=(Univariate&& other)
    {
        evaluations = std::move(other.evaluations);
        return *this;
    }

    bool operator==(const Univariate& other) const = default;

    Univariate operator+=(const Univariate& other)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] += other.evaluations[i];
        }
        return *this;
    }
    Univariate operator-=(const Univariate& other)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] -= other.evaluations[i];
        }
        return *this;
    }
    Univariate operator*=(const Univariate& other)
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
    Univariate operator*(const Univariate& other) const
    {
        Univariate res(*this);
        res *= other;
        return res;
    }

    // Operations between Univariate and scalar
    Univariate operator+=(const Fr& scalar)
    {
        for (auto& eval : evaluations) {
            eval += scalar;
        }
        return *this;
    }

    Univariate operator-=(const Fr& scalar)
    {
        for (auto& eval : evaluations) {
            eval -= scalar;
        }
        return *this;
    }
    Univariate operator*=(const Fr& scalar)
    {
        for (auto& eval : evaluations) {
            eval *= scalar;
        }
        return *this;
    }

    Univariate operator+(const Fr& scalar)
    {
        Univariate res(*this);
        res += scalar;
        return res;
    }

    Univariate operator-(const Fr& scalar)
    {
        Univariate res(*this);
        res -= scalar;
        return res;
    }

    Univariate operator*(const Fr& scalar)
    {
        Univariate res(*this);
        res *= scalar;
        return res;
    }

    // Operations between Univariate and UnivariateView
    Univariate operator+=(const UnivariateView<Fr, _length>& view)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] += view.evaluations[i];
        }
        return *this;
    }

    Univariate operator-=(const UnivariateView<Fr, _length>& view)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] -= view.evaluations[i];
        }
        return *this;
    }

    Univariate operator*=(const UnivariateView<Fr, _length>& view)
    {
        for (size_t i = 0; i < _length; ++i) {
            evaluations[i] *= view.evaluations[i];
        }
        return *this;
    }

    Univariate operator+(const UnivariateView<Fr, _length>& view)
    {
        Univariate res(*this);
        res += view;
        return res;
    }

    Univariate operator-(const UnivariateView<Fr, _length>& view)
    {
        Univariate res(*this);
        res -= view;
        return res;
    }

    Univariate operator*(const UnivariateView<Fr, _length>& view)
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

template <class Fr, size_t view_length> class UnivariateView {
  public:
    std::span<Fr, view_length> evaluations;

    UnivariateView() = default;

    Fr& value_at(size_t i) { return evaluations[i]; };

    template <size_t full_length>
    explicit UnivariateView(Univariate<Fr, full_length>& univariate_in)
        : evaluations(std::span<Fr>(univariate_in.evaluations.begin(), view_length)){};

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
};
} // namespace honk::sumcheck
