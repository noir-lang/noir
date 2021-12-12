#pragma once

namespace barretenberg {
namespace group_elements {
template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T>::element(const Fq& a, const Fq& b, const Fq& c) noexcept
    : x(a)
    , y(b)
    , z(c)
{}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T>::element(const element& other) noexcept
    : x(other.x)
    , y(other.y)
    , z(other.z)
{}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T>::element(element&& other) noexcept
    : x(other.x)
    , y(other.y)
    , z(other.z)
{}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T>::element(const affine_element<Fq, Fr, T>& other) noexcept
    : x(other.x)
    , y(other.y)
    , z(Fq::one())
{}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T>& element<Fq, Fr, T>::operator=(const element& other) noexcept
{
    x = other.x;
    y = other.y;
    z = other.z;
    return *this;
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T>& element<Fq, Fr, T>::operator=(element&& other) noexcept
{
    x = other.x;
    y = other.y;
    z = other.z;
    return *this;
}

template <class Fq, class Fr, class T> constexpr element<Fq, Fr, T>::operator affine_element<Fq, Fr, T>() const noexcept
{
    if (is_point_at_infinity()) {
        affine_element<Fq, Fr, T> result;
        result.x = Fq(0);
        result.y = Fq(0);
        result.self_set_infinity();
        return result;
    }
    Fq z_inv = z.invert();
    Fq zz_inv = z_inv.sqr();
    Fq zzz_inv = zz_inv * z_inv;
    affine_element<Fq, Fr, T> result(x * zz_inv, y * zzz_inv);
    if (is_point_at_infinity()) {
        result.self_set_infinity();
    }
    return result;
}

template <class Fq, class Fr, class T> constexpr void element<Fq, Fr, T>::self_dbl() noexcept
{
    if constexpr (Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        if (is_point_at_infinity()) {
            self_set_infinity();
            return;
        }
    } else {
        if (x.is_msb_set_word()) {
            self_set_infinity();
            return;
        }
    }

    // T0 = x*x
    Fq T0 = x.sqr();

    // T1 = y*y
    Fq T1 = y.sqr();

    // T2 = T2*T1 = y*y*y*y
    Fq T2 = T1.sqr();

    // T1 = T1 + x = x + y*y
    T1 += x;

    // T1 = T1 * T1
    T1.self_sqr();

    // T3 = T0 + T2 = xx + y*y*y*y
    Fq T3 = T0 + T2;

    // T1 = T1 - T3 = x*x + y*y*y*y + 2*x*x*y*y*y*y - x*x - y*y*y*y = 2*x*x*y*y*y*y = 2*S
    T1 -= T3;

    // T1 = 2T1 = 4*S
    T1 += T1;

    // T3 = 3T0
    T3 = T0 + T0;
    T3 += T0;
    if constexpr (T::has_a) {
        T3 += (T::a * z.sqr().sqr());
    }

    // z2 = 2*y*z
    z += z;
    z *= y;

    // T0 = 2T1
    T0 = T1 + T1;

    // x2 = T3*T3
    x = T3.sqr();

    // x2 = x2 - 2T1
    x -= T0;

    // T2 = 8T2
    T2 += T2;
    T2 += T2;
    T2 += T2;

    // y2 = T1 - x2
    y = T1 - x;

    // y2 = y2 * T3 - T2
    y *= T3;
    y -= T2;
}

template <class Fq, class Fr, class T> constexpr element<Fq, Fr, T> element<Fq, Fr, T>::dbl() const noexcept
{
    element result(*this);
    result.self_dbl();
    return result;
}

template <class Fq, class Fr, class T>
constexpr void element<Fq, Fr, T>::self_mixed_add_or_sub(const affine_element<Fq, Fr, T>& other,
                                                         const uint64_t predicate) noexcept
{
    if constexpr (Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        if (is_point_at_infinity()) {
            conditional_negate_affine(other, *(affine_element<Fq, Fr, T>*)this, predicate);
            z = Fq::one();
            return;
        }
    } else {
        const bool edge_case_trigger = x.is_msb_set() | other.x.is_msb_set();
        if (edge_case_trigger) {
            if (x.is_msb_set()) {
                conditional_negate_affine(other, *(affine_element<Fq, Fr, T>*)this, predicate);
                z = Fq::one();
            }
            return;
        }
    }

    // T0 = z1.z1
    Fq T0 = z.sqr();

    // T1 = x2.t0 - x1 = x2.z1.z1 - x1
    Fq T1 = other.x * T0;
    T1 -= x;

    // T2 = T0.z1 = z1.z1.z1
    // T2 = T2.y2 - y1 = y2.z1.z1.z1 - y1
    Fq T2 = z * T0;
    T2 *= other.y;
    T2.self_conditional_negate(predicate);
    T2 -= y;

    if (__builtin_expect(T1.is_zero(), 0)) {
        if (T2.is_zero()) {
            // y2 equals y1, x2 equals x1, double x1
            self_dbl();
            return;
        } else {
            self_set_infinity();
            return;
        }
    }

    // T2 = 2T2 = 2(y2.z1.z1.z1 - y1) = R
    // z3 = z1 + H
    T2 += T2;
    z += T1;

    // T3 = T1*T1 = HH
    Fq T3 = T1.sqr();

    // z3 = z3 - z1z1 - HH
    T0 += T3;

    // z3 = (z1 + H)*(z1 + H)
    z.self_sqr();
    z -= T0;

    // T3 = 4HH
    T3 += T3;
    T3 += T3;

    // T1 = T1*T3 = 4HHH
    T1 *= T3;

    // T3 = T3 * x1 = 4HH*x1
    T3 *= x;

    // T0 = 2T3
    T0 = T3 + T3;

    // T0 = T0 + T1 = 2(4HH*x1) + 4HHH
    T0 += T1;
    x = T2.sqr();

    // x3 = x3 - T0 = R*R - 8HH*x1 -4HHH
    x -= T0;

    // T3 = T3 - x3 = 4HH*x1 - x3
    T3 -= x;

    T1 *= y;
    T1 += T1;

    // T3 = T2 * T3 = R*(4HH*x1 - x3)
    T3 *= T2;

    // y3 = T3 - T1
    y = T3 - T1;
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator+=(const affine_element<Fq, Fr, T>& other) noexcept
{
    if constexpr (Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        if (is_point_at_infinity()) {
            *this = { other.x, other.y, Fq::one() };
            return *this;
        }
    } else {
        const bool edge_case_trigger = x.is_msb_set() | other.x.is_msb_set();
        if (edge_case_trigger) {
            if (x.is_msb_set()) {
                *this = { other.x, other.y, Fq::one() };
            }
            return *this;
        }
    }

    // T0 = z1.z1
    Fq T0 = z.sqr();

    // T1 = x2.t0 - x1 = x2.z1.z1 - x1
    Fq T1 = other.x * T0;
    T1 -= x;

    // T2 = T0.z1 = z1.z1.z1
    // T2 = T2.y2 - y1 = y2.z1.z1.z1 - y1
    Fq T2 = z * T0;
    T2 *= other.y;
    T2 -= y;

    if (__builtin_expect(T1.is_zero(), 0)) {
        if (T2.is_zero()) {
            self_dbl();
            return *this;
        } else {
            self_set_infinity();
            return *this;
        }
    }

    // T2 = 2T2 = 2(y2.z1.z1.z1 - y1) = R
    // z3 = z1 + H
    T2 += T2;
    z += T1;

    // T3 = T1*T1 = HH
    Fq T3 = T1.sqr();

    // z3 = z3 - z1z1 - HH
    T0 += T3;

    // z3 = (z1 + H)*(z1 + H)
    z.self_sqr();
    z -= T0;

    // T3 = 4HH
    T3 += T3;
    T3 += T3;

    // T1 = T1*T3 = 4HHH
    T1 *= T3;

    // T3 = T3 * x1 = 4HH*x1
    T3 *= x;

    // T0 = 2T3
    T0 = T3 + T3;

    // T0 = T0 + T1 = 2(4HH*x1) + 4HHH
    T0 += T1;
    x = T2.sqr();

    // x3 = x3 - T0 = R*R - 8HH*x1 -4HHH
    x -= T0;

    // T3 = T3 - x3 = 4HH*x1 - x3
    T3 -= x;

    T1 *= y;
    T1 += T1;

    // T3 = T2 * T3 = R*(4HH*x1 - x3)
    T3 *= T2;

    // y3 = T3 - T1
    y = T3 - T1;
    return *this;
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator+(const affine_element<Fq, Fr, T>& other) const noexcept
{
    element result(*this);
    return (result += other);
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator-=(const affine_element<Fq, Fr, T>& other) noexcept
{
    const affine_element<Fq, Fr, T> to_add{ other.x, -other.y };
    return operator+=(to_add);
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator-(const affine_element<Fq, Fr, T>& other) const noexcept
{
    element result(*this);
    return (result -= other);
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator+=(const element& other) noexcept
{
    if constexpr (Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        bool p1_zero = is_point_at_infinity();
        bool p2_zero = other.is_point_at_infinity();
        if (__builtin_expect((p1_zero || p2_zero), 0)) {
            if (p1_zero && !p2_zero) {
                *this = other;
                return *this;
            }
            if (p2_zero && !p1_zero) {
                return *this;
            }
            self_set_infinity();
            return *this;
        }
    } else {
        bool p1_zero = x.is_msb_set();
        bool p2_zero = other.x.is_msb_set();
        if (__builtin_expect((p1_zero || p2_zero), 0)) {
            if (p1_zero && !p2_zero) {
                *this = other;
                return *this;
            }
            if (p2_zero && !p1_zero) {
                return *this;
            }
            self_set_infinity();
            return *this;
        }
    }
    Fq Z1Z1(z.sqr());
    Fq Z2Z2(other.z.sqr());
    Fq S2(Z1Z1 * z);
    Fq U2(Z1Z1 * other.x);
    S2 *= other.y;
    Fq U1(Z2Z2 * x);
    Fq S1(Z2Z2 * other.z);
    S1 *= y;

    Fq F(S2 - S1);

    Fq H(U2 - U1);

    if (__builtin_expect(H.is_zero(), 0)) {
        if (F.is_zero()) {
            self_dbl();
            return *this;
        } else {
            self_set_infinity();
            return *this;
        }
    }

    F += F;

    Fq I(H + H);
    I.self_sqr();

    Fq J(H * I);

    U1 *= I;

    U2 = U1 + U1;
    U2 += J;

    x = F.sqr();

    x -= U2;

    J *= S1;
    J += J;

    y = U1 - x;

    y *= F;

    y -= J;

    z += other.z;

    Z1Z1 += Z2Z2;

    z.self_sqr();
    z -= Z1Z1;
    z *= H;
    return *this;
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator+(const element& other) const noexcept
{
    element result(*this);
    return (result += other);
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator-=(const element& other) noexcept
{
    const element to_add{ other.x, -other.y, other.z };
    return operator+=(to_add);
}

template <class Fq, class Fr, class T>
constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator-(const element& other) const noexcept
{
    element result(*this);
    return (result -= other);
}

template <class Fq, class Fr, class T> constexpr element<Fq, Fr, T> element<Fq, Fr, T>::operator-() const noexcept
{
    return { x, -y, z };
}

template <class Fq, class Fr, class T>
element<Fq, Fr, T> element<Fq, Fr, T>::operator*(const Fr& exponent) const noexcept
{
    if constexpr (T::USE_ENDOMORPHISM) {
        return mul_with_endomorphism(exponent);
    }
    return mul_without_endomorphism(exponent);
}

template <class Fq, class Fr, class T> element<Fq, Fr, T> element<Fq, Fr, T>::operator*=(const Fr& exponent) noexcept
{
    *this = operator*(exponent);
    return *this;
}

template <class Fq, class Fr, class T> constexpr element<Fq, Fr, T> element<Fq, Fr, T>::normalize() const noexcept
{
    const affine_element<Fq, Fr, T> converted = *this;
    return element(converted);
}

template <class Fq, class Fr, class T> constexpr element<Fq, Fr, T> element<Fq, Fr, T>::set_infinity() const noexcept
{
    element result(*this);
    result.self_set_infinity();
    return result;
}

template <class Fq, class Fr, class T> constexpr void element<Fq, Fr, T>::self_set_infinity() noexcept
{
    if constexpr (Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        x.data[0] = 0;
        x.data[1] = 0;
        x.data[2] = 0;
        x.data[3] = 0;
    } else {
        x.self_set_msb();
    }
}

template <class Fq, class Fr, class T> constexpr bool element<Fq, Fr, T>::is_point_at_infinity() const noexcept
{
    if constexpr (Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        return ((x.data[0] | x.data[1] | x.data[2] | x.data[3]) == 0);
    } else {
        return (x.is_msb_set());
    }
}

template <class Fq, class Fr, class T> constexpr bool element<Fq, Fr, T>::on_curve() const noexcept
{
    if (is_point_at_infinity()) {
        return true;
    }
    Fq zz = z.sqr();
    Fq zzzz = zz.sqr();
    Fq bz_6 = zzzz * zz * T::b;
    if constexpr (T::has_a) {
        bz_6 += (x * T::a) * zzzz;
    }
    Fq xxx = x.sqr() * x + bz_6;
    Fq yy = y.sqr();
    return (xxx == yy);
}

template <class Fq, class Fr, class T>
constexpr bool element<Fq, Fr, T>::operator==(const element& other) const noexcept
{
    bool both_infinity = is_point_at_infinity() && other.is_point_at_infinity();

    const Fq lhs_zz = z.sqr();
    const Fq lhs_zzz = lhs_zz * z;
    const Fq rhs_zz = other.z.sqr();
    const Fq rhs_zzz = rhs_zz * other.z;

    const Fq lhs_x = x * rhs_zz;
    const Fq lhs_y = y * rhs_zzz;

    const Fq rhs_x = other.x * lhs_zz;
    const Fq rhs_y = other.y * lhs_zzz;
    return both_infinity || ((lhs_x == rhs_x) && (lhs_y == rhs_y));
}

template <class Fq, class Fr, class T>
element<Fq, Fr, T> element<Fq, Fr, T>::random_element(numeric::random::Engine* engine) noexcept
{
    if constexpr (T::can_hash_to_curve) {
        element result = random_coordinates_on_curve(engine);
        result.z = Fq::random_element(engine);
        Fq zz = result.z.sqr();
        Fq zzz = zz * result.z;
        result.x *= zz;
        result.y *= zzz;
        return result;
    } else {
        Fr scalar = Fr::random_element(engine);
        return (element{ T::one_x, T::one_y, Fq::one() } * scalar);
    }
}

template <class Fq, class Fr, class T>
element<Fq, Fr, T> element<Fq, Fr, T>::mul_without_endomorphism(const Fr& exponent) const noexcept
{
    const Fr converted_scalar = exponent.from_montgomery_form();

    if (converted_scalar.is_zero()) {
        element result{ Fq::zero(), Fq::zero(), Fq::zero() };
        result.self_set_infinity();
        return result;
    }

    element work_element(*this);

    const uint64_t maximum_set_bit = converted_scalar.get_msb();
    for (uint64_t i = maximum_set_bit - 1; i < maximum_set_bit; --i) {
        work_element.self_dbl();
        if (converted_scalar.get_bit(i)) {
            work_element += *this;
        }
    }
    return work_element;
}

template <class Fq, class Fr, class T>
element<Fq, Fr, T> element<Fq, Fr, T>::mul_with_endomorphism(const Fr& exponent) const noexcept
{
    const Fr converted_scalar = exponent.from_montgomery_form();

    if (converted_scalar.is_zero()) {
        element result{ Fq::zero(), Fq::zero(), Fq::zero() };
        result.self_set_infinity();
        return result;
    }

    constexpr size_t lookup_size = 8;
    constexpr size_t num_rounds = 32;
    constexpr size_t num_wnaf_bits = 4;
    std::array<element, lookup_size> lookup_table;

    element d2 = element(*this);
    d2.self_dbl();
    lookup_table[0] = element(*this);
    for (size_t i = 1; i < lookup_size; ++i) {
        lookup_table[i] = lookup_table[i - 1] + d2;
    }

    uint64_t wnaf_table[num_rounds * 2];
    Fr endo_scalar;
    Fr::split_into_endomorphism_scalars(converted_scalar, endo_scalar, *(Fr*)&endo_scalar.data[2]);

    bool skew = false;
    bool endo_skew = false;

    wnaf::fixed_wnaf(&endo_scalar.data[0], &wnaf_table[0], skew, 0, 2, num_wnaf_bits);
    wnaf::fixed_wnaf(&endo_scalar.data[2], &wnaf_table[1], endo_skew, 0, 2, num_wnaf_bits);

    element work_element{ T::one_x, T::one_y, Fq::one() };
    work_element.self_set_infinity();

    uint64_t wnaf_entry;
    uint64_t index;
    bool sign;

    for (size_t i = 0; i < num_rounds * 2; ++i) {
        wnaf_entry = wnaf_table[i];
        index = wnaf_entry & 0x0fffffffU;
        sign = static_cast<bool>((wnaf_entry >> 31) & 1);
        const bool is_odd = ((i & 1) == 1);
        auto to_add = lookup_table[static_cast<size_t>(index)];
        to_add.y.self_conditional_negate(sign ^ is_odd);
        if (is_odd) {
            to_add.x *= Fq::beta();
        }
        work_element += to_add;

        if (i != ((2 * num_rounds) - 1) && is_odd) {
            for (size_t j = 0; j < 4; ++j) {
                work_element.self_dbl();
            }
        }
    }

    auto temporary = -lookup_table[0];
    if (skew) {
        work_element += temporary;
    }

    temporary = { lookup_table[0].x * Fq::beta(), lookup_table[0].y, lookup_table[0].z };

    if (endo_skew) {
        work_element += temporary;
    }

    return work_element;
}

template <class Fq, class Fr, class T>
std::vector<affine_element<Fq, Fr, T>> element<Fq, Fr, T>::batch_mul_with_endomorphism(
    const std::vector<affine_element<Fq, Fr, T>>& points, const Fr& exponent) noexcept
{
    typedef affine_element<Fq, Fr, T> affine_element;
    const size_t num_points = points.size();
    std::vector<Fq> scratch_space(num_points);

    // we can mutate rhs but NOT lhs!
    // output is stored in rhs
    const auto batch_affine_add = [num_points, &scratch_space](const affine_element* lhs, affine_element* rhs) {
        Fq batch_inversion_accumulator = Fq::one();

        for (size_t i = 0; i < num_points; i += 1) {
            scratch_space[i] = lhs[i].x + rhs[i].x;  // x2 + x1
            rhs[i].x -= lhs[i].x;                    // x2 - x1
            rhs[i].y -= lhs[i].y;                    // y2 - y1
            rhs[i].y *= batch_inversion_accumulator; // (y2 - y1)*accumulator_old
            batch_inversion_accumulator *= (rhs[i].x);
        }
        batch_inversion_accumulator = batch_inversion_accumulator.invert();

        for (size_t i = (num_points)-1; i < num_points; i -= 1) {
            rhs[i].y *= batch_inversion_accumulator; // update accumulator
            batch_inversion_accumulator *= rhs[i].x;
            rhs[i].x = rhs[i].y.sqr();
            rhs[i].x = rhs[i].x - (scratch_space[i]); // x3 = lambda_squared - x2
                                                      // - x1
            scratch_space[i] = lhs[i].x - rhs[i].x;
            scratch_space[i] *= rhs[i].y;
            rhs[i].y = scratch_space[i] - lhs[i].y;
        }
    };

    // double the elements in lhs
    const auto batch_affine_double = [num_points, &scratch_space](affine_element* lhs) {
        Fq batch_inversion_accumulator = Fq::one();

        for (size_t i = 0; i < num_points; i += 1) {

            scratch_space[i] = lhs[i].x.sqr();
            scratch_space[i] = scratch_space[i] + scratch_space[i] + scratch_space[i];

            scratch_space[i] *= batch_inversion_accumulator;

            batch_inversion_accumulator *= (lhs[i].y + lhs[i].y);
        }
        batch_inversion_accumulator = batch_inversion_accumulator.invert();

        Fq temp;
        for (size_t i = (num_points)-1; i < num_points; i -= 1) {

            scratch_space[i] *= batch_inversion_accumulator;
            batch_inversion_accumulator *= (lhs[i].y + lhs[i].y);

            temp = lhs[i].x;
            lhs[i].x = scratch_space[i].sqr() - (lhs[i].x + lhs[i].x);
            lhs[i].y = scratch_space[i] * (temp - lhs[i].x) - lhs[i].y;
        }
    };

    // Compute wnaf for scalar
    const Fr converted_scalar = exponent.from_montgomery_form();

    if (converted_scalar.is_zero()) {
        affine_element result{ Fq::zero(), Fq::zero() };
        result.self_set_infinity();
        std::vector<affine_element> results;
        for (size_t i = 0; i < num_points; ++i) {
            results.emplace_back(result);
        }
        return results;
    }

    constexpr size_t lookup_size = 8;
    constexpr size_t num_rounds = 32;
    constexpr size_t num_wnaf_bits = 4;
    std::array<std::vector<affine_element>, lookup_size> lookup_table;
    for (auto& table : lookup_table) {
        table.resize(num_points);
    }
    std::vector<affine_element> temp_point_vector(num_points);
    for (size_t i = 0; i < num_points; ++i) {
        temp_point_vector[i] = points[i];
        lookup_table[0][i] = points[i];
    }
    batch_affine_double(&temp_point_vector[0]);
    for (size_t j = 1; j < lookup_size; ++j) {

        for (size_t i = 0; i < num_points; ++i) {
            lookup_table[j][i] = lookup_table[j - 1][i];
        }
        batch_affine_add(&temp_point_vector[0], &lookup_table[j][0]);
    }

    uint64_t wnaf_table[num_rounds * 2];
    Fr endo_scalar;
    Fr::split_into_endomorphism_scalars(converted_scalar, endo_scalar, *(Fr*)&endo_scalar.data[2]);

    bool skew = false;
    bool endo_skew = false;

    wnaf::fixed_wnaf(&endo_scalar.data[0], &wnaf_table[0], skew, 0, 2, num_wnaf_bits);
    wnaf::fixed_wnaf(&endo_scalar.data[2], &wnaf_table[1], endo_skew, 0, 2, num_wnaf_bits);

    std::vector<affine_element> work_elements(num_points);

    uint64_t wnaf_entry;
    uint64_t index;
    bool sign;
    for (size_t i = 0; i < 2; ++i) {
        for (size_t j = 0; j < num_points; ++j) {
            wnaf_entry = wnaf_table[i];
            index = wnaf_entry & 0x0fffffffU;
            sign = static_cast<bool>((wnaf_entry >> 31) & 1);
            const bool is_odd = ((i & 1) == 1);
            auto to_add = lookup_table[static_cast<size_t>(index)][j];
            to_add.y.self_conditional_negate(sign ^ is_odd);
            if (is_odd) {
                to_add.x *= Fq::beta();
            }
            if (i == 0) {
                work_elements[j] = to_add;
            } else {
                temp_point_vector[j] = to_add;
            }
        }
    }
    batch_affine_add(&temp_point_vector[0], &work_elements[0]);

    for (size_t i = 2; i < num_rounds * 2; ++i) {
        wnaf_entry = wnaf_table[i];
        index = wnaf_entry & 0x0fffffffU;
        sign = static_cast<bool>((wnaf_entry >> 31) & 1);
        const bool is_odd = ((i & 1) == 1);
        if (!is_odd) {
            for (size_t k = 0; k < 4; ++k) {
                batch_affine_double(&work_elements[0]);
            }
        }
        for (size_t j = 0; j < num_points; ++j) {
            auto to_add = lookup_table[static_cast<size_t>(index)][j];
            to_add.y.self_conditional_negate(sign ^ is_odd);
            if (is_odd) {
                to_add.x *= Fq::beta();
            }
            temp_point_vector[j] = to_add;
        }
        batch_affine_add(&temp_point_vector[0], &work_elements[0]);
    }

    if (skew) {
        for (size_t j = 0; j < num_points; ++j) {
            temp_point_vector[j] = -lookup_table[0][j];
        }
        batch_affine_add(&temp_point_vector[0], &work_elements[0]);
    }

    if (endo_skew) {
        for (size_t j = 0; j < num_points; ++j) {
            temp_point_vector[j] = lookup_table[0][j];
            temp_point_vector[j].x *= Fq::beta();
        }
        batch_affine_add(&temp_point_vector[0], &work_elements[0]);
    }

    return work_elements;
}

template <typename Fq, typename Fr, typename T>
void element<Fq, Fr, T>::conditional_negate_affine(const affine_element<Fq, Fr, T>& src,
                                                   affine_element<Fq, Fr, T>& dest,
                                                   const uint64_t predicate) noexcept
{
    dest = { src.x, predicate ? -src.y : src.y };
}

template <typename Fq, typename Fr, typename T>
void element<Fq, Fr, T>::batch_normalize(element* elements, const size_t num_elements) noexcept
{
    std::vector<Fq> temporaries;
    temporaries.reserve(num_elements * 2);
    Fq accumulator = Fq::one();

    // Iterate over the points, computing the product of their z-coordinates.
    // At each iteration, store the currently-accumulated z-coordinate in `temporaries`
    for (size_t i = 0; i < num_elements; ++i) {
        temporaries.emplace_back(accumulator);
        if (!elements[i].is_point_at_infinity()) {
            accumulator *= elements[i].z;
        }
    }
    // For the rest of this method we refer to the product of all z-coordinates as the 'global' z-coordinate
    // Invert the global z-coordinate and store in `accumulator`
    accumulator = accumulator.invert();

    /**
     * We now proceed to iterate back down the array of points.
     * At each iteration we update the accumulator to contain the z-coordinate of the currently worked-upon
     *z-coordinate. We can then multiply this accumulator with `temporaries`, to get a scalar that is equal to the
     *inverse of the z-coordinate of the point at the next iteration cycle e.g. Imagine we have 4 points, such that:
     *
     * accumulator = 1 / z.data[0]*z.data[1]*z.data[2]*z.data[3]
     * temporaries[3] = z.data[0]*z.data[1]*z.data[2]
     * temporaries[2] = z.data[0]*z.data[1]
     * temporaries[1] = z.data[0]
     * temporaries[0] = 1
     *
     * At the first iteration, accumulator * temporaries[3] = z.data[0]*z.data[1]*z.data[2] /
     *z.data[0]*z.data[1]*z.data[2]*z.data[3]  = (1 / z.data[3]) We then update accumulator, such that:
     *
     * accumulator = accumulator * z.data[3] = 1 / z.data[0]*z.data[1]*z.data[2]
     *
     * At the second iteration, accumulator * temporaries[2] = z.data[0]*z.data[1] / z.data[0]*z.data[1]*z.data[2] =
     *(1 z.data[2]) And so on, until we have computed every z-inverse!
     *
     * We can then convert out of Jacobian form (x = X / Z^2, y = Y / Z^3) with 4 muls and 1 square.
     **/
    for (size_t i = num_elements - 1; i < num_elements; --i) {
        if (!elements[i].is_point_at_infinity()) {
            Fq z_inv = accumulator * temporaries[i];
            Fq zz_inv = z_inv.sqr();
            elements[i].x *= zz_inv;
            elements[i].y *= (zz_inv * z_inv);
            accumulator *= elements[i].z;
        }
        elements[i].z = Fq::one();
    }
}

template <typename Fq, typename Fr, typename T>
template <typename>
element<Fq, Fr, T> element<Fq, Fr, T>::random_coordinates_on_curve(numeric::random::Engine* engine) noexcept
{
    bool found_one = false;
    Fq yy;
    Fq x;
    Fq y;
    Fq t0;
    while (!found_one) {
        x = Fq::random_element(engine);
        yy = x.sqr() * x + T::b;
        if constexpr (T::has_a) {
            yy += (x * T::a);
        }
        y = yy.sqrt();
        t0 = y.sqr();
        found_one = (yy == t0);
    }
    return { x, y, Fq::one() };
}

} // namespace group_elements
} // namespace barretenberg