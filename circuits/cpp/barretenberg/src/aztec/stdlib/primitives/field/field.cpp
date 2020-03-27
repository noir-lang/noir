#include "field.hpp"
#include "../bool/bool.hpp"
#include "../composers/composers.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext>
field_t<ComposerContext>::field_t(ComposerContext* parent_context)
    : context(parent_context)
    , additive_constant(barretenberg::fr::zero())
    , multiplicative_constant(barretenberg::fr::one())
    , witness_index(static_cast<uint32_t>(-1))
{}

template <typename ComposerContext>
field_t<ComposerContext>::field_t(const witness_t<ComposerContext>& value)
    : context(value.context)
{
    additive_constant = barretenberg::fr::zero();
    multiplicative_constant = barretenberg::fr::one();
    witness_index = value.witness_index;
}

template <typename ComposerContext>
field_t<ComposerContext>::field_t(ComposerContext* parent_context, const barretenberg::fr& value)
    : context(parent_context)
{
    barretenberg::fr::__copy(value, additive_constant);
    multiplicative_constant = barretenberg::fr::zero();
    witness_index = static_cast<uint32_t>(-1);
}

template <typename ComposerContext> field_t<ComposerContext>::field_t(const bool_t<ComposerContext>& other)
{
    context = (other.context == nullptr) ? nullptr : other.context;
    if (other.witness_index == static_cast<uint32_t>(-1)) {
        additive_constant =
            (other.witness_bool ^ other.witness_inverted) ? barretenberg::fr::one() : barretenberg::fr::zero();
        multiplicative_constant = barretenberg::fr::one();
        witness_index = static_cast<uint32_t>(-1);
    } else {
        witness_index = other.witness_index;
        additive_constant = other.witness_inverted ? barretenberg::fr::one() : barretenberg::fr::zero();
        multiplicative_constant = other.witness_inverted ? barretenberg::fr::neg_one() : barretenberg::fr::one();
    }
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::from_witness_index(ComposerContext* ctx,
                                                                      const uint32_t witness_index)
{
    field_t<ComposerContext> result(ctx);
    result.witness_index = witness_index;
    return result;
}

template <typename ComposerContext> field_t<ComposerContext>::operator bool_t<ComposerContext>()
{
    if (witness_index == static_cast<uint32_t>(-1)) {
        bool_t<ComposerContext> result(context);
        result.witness_bool = (additive_constant == barretenberg::fr::one());
        result.witness_inverted = false;
        result.witness_index = static_cast<uint32_t>(-1);
        return result;
    }
    bool add_constant_check = (additive_constant == barretenberg::fr::zero());
    bool mul_constant_check = (multiplicative_constant == barretenberg::fr::one());
    bool inverted_check =
        (additive_constant == barretenberg::fr::one()) && (multiplicative_constant == barretenberg::fr::neg_one());
    if ((!add_constant_check || !mul_constant_check) && !inverted_check) {
        normalize();
    }

    barretenberg::fr witness = context->get_variable(witness_index);
    ASSERT((witness == barretenberg::fr::zero()) || (witness == barretenberg::fr::one()));
    bool_t<ComposerContext> result(context);
    result.witness_bool = (witness == barretenberg::fr::one());
    result.witness_inverted = inverted_check;
    result.witness_index = witness_index;
    context->create_bool_gate(witness_index);
    return result;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::operator+(const field_t& other) const
{
    ComposerContext* ctx = (context == nullptr) ? other.context : context;
    field_t<ComposerContext> result(ctx);
    ASSERT(ctx || (witness_index == static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)));

    if (witness_index == other.witness_index) {
        result.additive_constant = additive_constant + other.additive_constant;
        result.multiplicative_constant = multiplicative_constant + other.multiplicative_constant;
        result.witness_index = witness_index;
    } else if (witness_index == static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)) {
        // both inputs are constant - don't add a gate
        result.additive_constant = additive_constant + other.additive_constant;
    } else if (witness_index != static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)) {
        // one input is constant - don't add a gate, but update scaling factors
        result.additive_constant = additive_constant + other.additive_constant;
        barretenberg::fr::__copy(multiplicative_constant, result.multiplicative_constant);
        result.witness_index = witness_index;
    } else if (witness_index == static_cast<uint32_t>(-1) && other.witness_index != static_cast<uint32_t>(-1)) {
        result.additive_constant = additive_constant + other.additive_constant;
        barretenberg::fr::__copy(other.multiplicative_constant, result.multiplicative_constant);
        result.witness_index = other.witness_index;
    } else {
        barretenberg::fr T0;
        barretenberg::fr left = context->get_variable(witness_index);
        barretenberg::fr right = context->get_variable(other.witness_index);
        barretenberg::fr out;
        out = left * multiplicative_constant;
        T0 = right * other.multiplicative_constant;
        out += T0;
        out += additive_constant;
        out += other.additive_constant;
        result.witness_index = ctx->add_variable(out);

        const waffle::add_triple gate_coefficients{ witness_index,
                                                    other.witness_index,
                                                    result.witness_index,
                                                    multiplicative_constant,
                                                    other.multiplicative_constant,
                                                    barretenberg::fr::neg_one(),
                                                    (additive_constant + other.additive_constant) };
        ctx->create_add_gate(gate_coefficients);
    }
    return result;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::operator-(const field_t& other) const
{
    field_t<ComposerContext> rhs(other);
    rhs.additive_constant.self_neg();
    rhs.multiplicative_constant.self_neg();
    return operator+(rhs);
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::operator*(const field_t& other) const
{
    ComposerContext* ctx = (context == nullptr) ? other.context : context;
    field_t<ComposerContext> result(ctx);
    ASSERT(ctx || (witness_index == static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)));

    if (witness_index == static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)) {
        // both inputs are constant - don't add a gate
        result.additive_constant = additive_constant * other.additive_constant;
    } else if (witness_index != static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)) {
        // one input is constant - don't add a gate, but update scaling factors
        result.additive_constant = additive_constant * other.additive_constant;
        result.multiplicative_constant = multiplicative_constant * other.additive_constant;
        result.witness_index = witness_index;
    } else if (witness_index == static_cast<uint32_t>(-1) && other.witness_index != static_cast<uint32_t>(-1)) {
        result.additive_constant = additive_constant * other.additive_constant;
        result.multiplicative_constant = other.multiplicative_constant * additive_constant;
        result.witness_index = other.witness_index;
    } else {
        // both inputs map to circuit varaibles - create a * constraint
        barretenberg::fr T0;
        barretenberg::fr q_m;
        barretenberg::fr q_l;
        barretenberg::fr q_r;
        barretenberg::fr q_c;

        q_c = additive_constant * other.additive_constant;
        q_r = additive_constant * other.multiplicative_constant;
        q_l = multiplicative_constant * other.additive_constant;
        q_m = multiplicative_constant * other.multiplicative_constant;

        barretenberg::fr left = context->get_variable(witness_index);
        barretenberg::fr right = context->get_variable(other.witness_index);
        barretenberg::fr out;

        out = left * right;
        out *= q_m;
        T0 = left * q_l;
        out += T0;
        T0 = right * q_r;
        out += T0;
        out += q_c;
        result.witness_index = ctx->add_variable(out);
        const waffle::poly_triple gate_coefficients{
            witness_index, other.witness_index, result.witness_index, q_m, q_l, q_r, barretenberg::fr::neg_one(), q_c
        };
        ctx->create_poly_gate(gate_coefficients);
    }
    return result;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::operator/(const field_t& other) const
{
    ComposerContext* ctx = (context == nullptr) ? other.context : context;
    field_t<ComposerContext> result(ctx);
    ASSERT(ctx || (witness_index == static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)));

    barretenberg::fr additive_multiplier = barretenberg::fr::one();

    if (witness_index == static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)) {
        // both inputs are constant - don't add a gate
        if (!(other.additive_constant == barretenberg::fr::zero())) {
            additive_multiplier = other.additive_constant.invert();
        }
        result.additive_constant = additive_constant * additive_multiplier;
    } else if (witness_index != static_cast<uint32_t>(-1) && other.witness_index == static_cast<uint32_t>(-1)) {
        // one input is constant - don't add a gate, but update scaling factors
        if (!(other.additive_constant == barretenberg::fr::zero())) {
            additive_multiplier = other.additive_constant.invert();
        }
        result.additive_constant = additive_constant * additive_multiplier;
        result.multiplicative_constant = multiplicative_constant * additive_multiplier;
        result.witness_index = witness_index;
    } else if (witness_index == static_cast<uint32_t>(-1) && other.witness_index != static_cast<uint32_t>(-1)) {
        if (!(other.additive_constant == barretenberg::fr::zero())) {
            additive_multiplier = other.additive_constant.invert();
        }
        result.additive_constant = additive_constant * other.additive_constant;
        result.multiplicative_constant = other.multiplicative_constant * additive_constant;
        result.witness_index = other.witness_index;
    } else {
        barretenberg::fr left = context->get_variable(witness_index);
        barretenberg::fr right = context->get_variable(other.witness_index);
        barretenberg::fr out;

        // even if LHS is constant, if divisor is not constant we need a gate to compute the inverse
        // barretenberg::fr witness_multiplier = other.witness.invert();
        // m1.x1 + a1 / (m2.x2 + a2) = x3
        barretenberg::fr T0;
        T0 = multiplicative_constant * left;
        T0 += additive_constant;
        barretenberg::fr T1;
        T1 = other.multiplicative_constant * right;
        T1 += other.additive_constant;

        out = T0 * T1.invert();
        result.witness_index = ctx->add_variable(out);

        // m2.x2.x3 + a2.x3 = m1.x1 + a1
        // m2.x2.x3 + a2.x3 - m1.x1 - a1 = 0
        // left = x3
        // right = x2
        // out = x1
        // qm = m2
        // ql = a2
        // qr = 0
        // qo = -m1
        // qc = -a1
        barretenberg::fr q_m = other.multiplicative_constant;
        barretenberg::fr q_l = other.additive_constant;
        barretenberg::fr q_r = barretenberg::fr::zero();
        barretenberg::fr q_o = -multiplicative_constant;
        barretenberg::fr q_c = -additive_constant;

        const waffle::poly_triple gate_coefficients{
            result.witness_index, other.witness_index, witness_index, q_m, q_l, q_r, q_o, q_c
        };
        ctx->create_poly_gate(gate_coefficients);
    }
    return result;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::madd(const field_t& to_mul, const field_t& to_add) const
{
    ComposerContext* ctx =
        (context == nullptr) ? (to_mul.context == nullptr ? to_add.context : to_mul.context) : context;

    if ((to_mul.witness_index == UINT32_MAX) || (to_add.witness_index == UINT32_MAX) || (witness_index == UINT32_MAX)) {
        return ((*this) * to_mul + to_add).normalize();
    }

    // (a * Q_a  + R_a) * (b * Q_b + R_b) + (c * Q_c  R_c) = result
    barretenberg::fr q_m = multiplicative_constant * to_mul.multiplicative_constant;
    barretenberg::fr q_1 = multiplicative_constant * to_mul.additive_constant;
    barretenberg::fr q_2 = to_mul.multiplicative_constant * additive_constant;
    barretenberg::fr q_3 = to_add.multiplicative_constant;
    barretenberg::fr q_c = additive_constant * to_mul.additive_constant + to_add.additive_constant;

    barretenberg::fr a = ctx->get_variable(witness_index);
    barretenberg::fr b = ctx->get_variable(to_mul.witness_index);
    barretenberg::fr c = ctx->get_variable(to_add.witness_index);

    barretenberg::fr out = a * b * q_m + a * q_1 + b * q_2 + c * q_3 + q_c;

    field_t<ComposerContext> result(ctx);
    result.witness_index = ctx->add_variable(out);

    const waffle::mul_quad gate_coefficients{
        witness_index,
        to_mul.witness_index,
        to_add.witness_index,
        result.witness_index,
        q_m,
        q_1,
        q_2,
        q_3,
        -barretenberg::fr(1),
        q_c,
    };
    ctx->create_big_mul_gate(gate_coefficients);
    return result;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::add_two(const field_t& add_a, const field_t& add_b) const
{
    ComposerContext* ctx = (context == nullptr) ? (add_a.context == nullptr ? add_b.context : add_a.context) : context;

    if ((add_a.witness_index == UINT32_MAX) || (add_b.witness_index == UINT32_MAX) || (witness_index == UINT32_MAX)) {
        return ((*this) + add_a + add_b).normalize();
    }

    barretenberg::fr q_1 = multiplicative_constant;
    barretenberg::fr q_2 = add_a.multiplicative_constant;
    barretenberg::fr q_3 = add_b.multiplicative_constant;
    barretenberg::fr q_c = additive_constant + add_a.additive_constant + add_b.additive_constant;

    barretenberg::fr a = ctx->get_variable(witness_index);
    barretenberg::fr b = ctx->get_variable(add_a.witness_index);
    barretenberg::fr c = ctx->get_variable(add_b.witness_index);

    barretenberg::fr out = a * q_1 + b * q_2 + c * q_3 + q_c;

    field_t<ComposerContext> result(ctx);
    result.witness_index = ctx->add_variable(out);

    const waffle::mul_quad gate_coefficients{
        witness_index,
        add_a.witness_index,
        add_b.witness_index,
        result.witness_index,
        barretenberg::fr(0),
        q_1,
        q_2,
        q_3,
        -barretenberg::fr(1),
        q_c,
    };
    ctx->create_big_mul_gate(gate_coefficients);
    return result;
}

template <typename ComposerContext> field_t<ComposerContext> field_t<ComposerContext>::normalize() const
{
    if (witness_index == static_cast<uint32_t>(-1) ||
        ((multiplicative_constant == barretenberg::fr::one()) && (additive_constant == barretenberg::fr::zero()))) {
        return *this;
    }

    field_t<ComposerContext> result(context);
    barretenberg::fr value = context->get_variable(witness_index);
    barretenberg::fr out;
    out = value * multiplicative_constant;
    out += additive_constant;

    result.witness_index = context->add_variable(out);
    result.additive_constant = barretenberg::fr::zero();
    result.multiplicative_constant = barretenberg::fr::one();
    const waffle::add_triple gate_coefficients{
        witness_index,    witness_index, result.witness_index, multiplicative_constant, 0, barretenberg::fr::neg_one(),
        additive_constant
    };

    context->create_add_gate(gate_coefficients);
    return result;
}

template <typename ComposerContext> bool_t<ComposerContext> field_t<ComposerContext>::is_zero()
{
    if (witness_index == static_cast<uint32_t>(-1)) {
        return bool_t(context, (get_value() == barretenberg::fr::zero()));
    }

    // To check whether a field element, k, is zero, we use the fact that, if k > 0,
    // there exists a modular inverse k', such that k * k' = 1

    // To verify whether k = 0, we must do 2 checks
    // First is that (k * k') - 1 + is_zero = 0

    // If is_zero = false, then k' must be the modular inverse of k, therefore k is not 0

    // If is_zero = true, then either k or k' is zero (or both)
    // To ensure that it is k that is zero, and not k', we must apply
    // an additional check: that if is_zero = true, k' = 1
    // This way, if (k * k') = 0, we know that k = 0.
    // The second check is: (is_zero * k') - is_zero = 0
    field_t k = normalize();
    bool_t is_zero = witness_t(context, (k.get_value() == barretenberg::fr::zero()));
    field_t k_inverse;
    if (is_zero.get_value()) {
        k_inverse = witness_t(context, barretenberg::fr::one());
    } else {
        barretenberg::fr k_inverse_value = k.get_value().invert();
        k_inverse = witness_t(context, k_inverse_value);
    }

    // k * k_inverse + is_zero - 1 = 0
    barretenberg::fr q_m = barretenberg::fr::one();
    barretenberg::fr q_l = barretenberg::fr::zero();
    barretenberg::fr q_r = barretenberg::fr::zero();
    barretenberg::fr q_o = barretenberg::fr::one();
    barretenberg::fr q_c = barretenberg::fr::neg_one();
    const waffle::poly_triple gate_coefficients_a{
        k.witness_index, k_inverse.witness_index, is_zero.witness_index, q_m, q_l, q_r, q_o, q_c
    };
    context->create_poly_gate(gate_coefficients_a);

    // is_zero * k_inverse - is_zero = 0
    q_o = barretenberg::fr::neg_one();
    q_c = barretenberg::fr::zero();
    const waffle::poly_triple gate_coefficients_b{
        is_zero.witness_index, k_inverse.witness_index, is_zero.witness_index, q_m, q_l, q_r, q_o, q_c
    };
    context->create_poly_gate(gate_coefficients_b);
    return is_zero;
}

template <typename ComposerContext> barretenberg::fr field_t<ComposerContext>::get_value() const
{
    if (witness_index != static_cast<uint32_t>(-1)) {
        ASSERT(context != nullptr);
        return (multiplicative_constant * context->get_variable(witness_index)) + additive_constant;
    } else {
        return additive_constant;
    }
}

template <typename ComposerContext>
bool_t<ComposerContext> field_t<ComposerContext>::operator==(const field_t& other) const
{
    ComposerContext* ctx = (context == nullptr) ? other.context : context;

    if (is_constant() && other.is_constant()) {
        return (get_value() == other.get_value());
    }

    barretenberg::fr fa = get_value();
    barretenberg::fr fb = other.get_value();
    barretenberg::fr fd = fa - fb;
    bool is_equal = (fa == fb);
    barretenberg::fr fc = is_equal ? barretenberg::fr::one() : fd.invert();

    bool_t result(witness_t(ctx, is_equal));
    field_t c(witness_t(ctx, fc));
    field_t d = *this - other;
    field_t test_lhs = d * c;
    field_t test_rhs = (field_t(ctx, barretenberg::fr::one()) - field_t(result));
    test_rhs = test_rhs.normalize();
    ctx->assert_equal(test_lhs.witness_index, test_rhs.witness_index);

    barretenberg::fr fe = is_equal ? barretenberg::fr::one() : fd;
    field_t e(witness_t(ctx, fe));

    // Ensures c is never 0.
    barretenberg::fr q_m = barretenberg::fr::one();
    barretenberg::fr q_l = barretenberg::fr::zero();
    barretenberg::fr q_r = barretenberg::fr::zero();
    barretenberg::fr q_c = barretenberg::fr::neg_one();
    barretenberg::fr q_o = barretenberg::fr::zero();
    const waffle::poly_triple gate_coefficients{
        c.witness_index, e.witness_index, c.witness_index, q_m, q_l, q_r, q_o, q_c
    };
    ctx->create_poly_gate(gate_coefficients);

    return result;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::conditional_negate(const bool_t<ComposerContext>& predicate) const
{
    field_t<ComposerContext> predicate_field(predicate);
    field_t<ComposerContext> multiplicand = -(predicate_field + predicate_field);
    return multiplicand.madd(*this, *this);
}

template <typename ComposerContext>
std::array<field_t<ComposerContext>, 4> field_t<ComposerContext>::preprocess_two_bit_table(const field_t& T0,
                                                                                           const field_t& T1,
                                                                                           const field_t& T2,
                                                                                           const field_t& T3)
{
    // (1 - t0)(1 - t1).T0 + t0(1 - t1).T1 + (1 - t0)t1.T2 + t0.t1.T3

    // -t0.t1.T0 - t0.t1.T1 -t0.t1.T2 + t0.t1.T3 => t0.t1(T3 - T2 - T1 + T0)
    // -t0.T0 + t0.T1 => t0(T1 - T0)
    // -t1.T0 - t1.T2 => t1(T2 - T0)
    // T0 = constant term
    std::array<field_t, 4> table;
    table[0] = T0;
    table[1] = T1 - T0;
    table[2] = T2 - T0;
    table[3] = T3 - T2 - T1 + T0;
    return table;
}

template <typename ComposerContext>
std::array<field_t<ComposerContext>, 8> field_t<ComposerContext>::preprocess_three_bit_table(const field_t& T0,
                                                                                             const field_t& T1,
                                                                                             const field_t& T2,
                                                                                             const field_t& T3,
                                                                                             const field_t& T4,
                                                                                             const field_t& T5,
                                                                                             const field_t& T6,
                                                                                             const field_t& T7)
{
    std::array<field_t, 8> table;
    table[0] = T0;                                    // const
    table[1] = T1 - T0;                               // t0
    table[2] = T2 - T0;                               // t1
    table[3] = T4 - T0;                               // t2
    table[4] = T3 - T2 - T1 + T0;                     // t0t1
    table[5] = T5 - T4 - T1 + T0;                     // t0t2
    table[6] = T6 - T4 - T2 + T0;                     // t1t2
    table[7] = T7 - T6 - T5 + T4 - T3 + T2 + T1 - T0; // t0t1t2
    return table;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::select_from_two_bit_table(const std::array<field_t, 4>& table,
                                                                             const bool_t<ComposerContext>& t1,
                                                                             const bool_t<ComposerContext>& t0)
{
    field_t R0 = static_cast<field_t>(t1).madd(table[3], table[1]);
    field_t R1 = R0.madd(static_cast<field_t>(t0), table[0]);
    field_t R2 = static_cast<field_t>(t1).madd(table[2], R1);
    return R2;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::select_from_three_bit_table(const std::array<field_t, 8>& table,
                                                                               const bool_t<ComposerContext>& t2,
                                                                               const bool_t<ComposerContext>& t1,
                                                                               const bool_t<ComposerContext>& t0)
{
    field_t R0 = static_cast<field_t>(t0).madd(table[7], table[6]);
    field_t R1 = static_cast<field_t>(t1).madd(R0, table[3]);
    field_t R2 = static_cast<field_t>(t2).madd(R1, table[0]);
    field_t R3 = static_cast<field_t>(t0).madd(table[4], table[2]);
    field_t R4 = static_cast<field_t>(t1).madd(R3, R2);
    field_t R5 = static_cast<field_t>(t2).madd(table[5], table[1]);
    field_t R6 = static_cast<field_t>(t0).madd(R5, R4);
    return R6;
}

template <typename ComposerContext>
void field_t<ComposerContext>::evaluate_polynomial_identity(const field_t& a,
                                                            const field_t& b,
                                                            const field_t& c,
                                                            const field_t& d)
{
    ComposerContext* ctx = a.context == nullptr
                               ? (b.context == nullptr ? (c.context == nullptr ? d.context : c.context) : b.context)
                               : a.context;

    if (a.witness_index == UINT32_MAX && b.witness_index == UINT32_MAX && c.witness_index == UINT32_MAX &&
        d.witness_index == UINT32_MAX) {
        return;
    }

    // validate that a * b + c + d = 0
    barretenberg::fr q_m = a.multiplicative_constant * b.multiplicative_constant;
    barretenberg::fr q_1 = a.multiplicative_constant * b.additive_constant;
    barretenberg::fr q_2 = b.multiplicative_constant * a.additive_constant;
    barretenberg::fr q_3 = c.multiplicative_constant;
    barretenberg::fr q_4 = d.multiplicative_constant;
    barretenberg::fr q_c = a.additive_constant * b.additive_constant + c.additive_constant + d.additive_constant;

    // debug TODO REMOVE

    barretenberg::fr t1 = a.get_value();
    barretenberg::fr t2 = b.get_value();
    barretenberg::fr t3 = c.get_value();
    barretenberg::fr t4 = d.get_value();
    if (t1 * t2 != -(t3 + t4)) {
        printf("polynomial identity does not validate!\n");
        std::cout << "t1 * t2 + t3 + t4 = " << (t1 * t2 + t3 + t4) << std::endl;
    }
    const waffle::mul_quad gate_coefficients{
        a.witness_index == UINT32_MAX ? ctx->zero_idx : a.witness_index,
        b.witness_index == UINT32_MAX ? ctx->zero_idx : b.witness_index,
        c.witness_index == UINT32_MAX ? ctx->zero_idx : c.witness_index,
        d.witness_index == UINT32_MAX ? ctx->zero_idx : d.witness_index,
        q_m,
        q_1,
        q_2,
        q_3,
        q_4,
        q_c,
    };
    ctx->create_big_mul_gate(gate_coefficients);
}

INSTANTIATE_STDLIB_TYPE(field_t);

} // namespace stdlib
} // namespace plonk
