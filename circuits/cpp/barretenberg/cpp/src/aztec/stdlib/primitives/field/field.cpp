#include "field.hpp"
#include <functional>
#include "../bool/bool.hpp"
#include "../composers/composers.hpp"
#include "../../../rollup/constants.hpp"
#include "../../../rollup/constants.hpp"

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"

namespace plonk {
namespace stdlib {

template <typename ComposerContext>
field_t<ComposerContext>::field_t(ComposerContext* parent_context)
    : context(parent_context)
    , additive_constant(barretenberg::fr::zero())
    , multiplicative_constant(barretenberg::fr::one())
    , witness_index(IS_CONSTANT)
{}

template <typename ComposerContext>
field_t<ComposerContext>::field_t(const witness_t<ComposerContext>& value)
    : context(value.context)
{
    additive_constant = 0;
    multiplicative_constant = 1;
    witness_index = value.witness_index;
}

template <typename ComposerContext>
field_t<ComposerContext>::field_t(ComposerContext* parent_context, const barretenberg::fr& value)
    : context(parent_context)
{
    additive_constant = value;
    multiplicative_constant = barretenberg::fr::zero();
    witness_index = IS_CONSTANT;
}

template <typename ComposerContext> field_t<ComposerContext>::field_t(const bool_t<ComposerContext>& other)
{
    context = (other.context == nullptr) ? nullptr : other.context;
    if (other.witness_index == IS_CONSTANT) {
        additive_constant =
            (other.witness_bool ^ other.witness_inverted) ? barretenberg::fr::one() : barretenberg::fr::zero();
        multiplicative_constant = barretenberg::fr::one();
        witness_index = IS_CONSTANT;
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

template <typename ComposerContext> field_t<ComposerContext>::operator bool_t<ComposerContext>() const
{
    if (witness_index == IS_CONSTANT) {
        bool_t<ComposerContext> result(context);
        result.witness_bool = (additive_constant == barretenberg::fr::one());
        result.witness_inverted = false;
        result.witness_index = IS_CONSTANT;
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
    ASSERT(ctx || (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT));

    if (witness_index == other.witness_index) {
        result.additive_constant = additive_constant + other.additive_constant;
        result.multiplicative_constant = multiplicative_constant + other.multiplicative_constant;
        result.witness_index = witness_index;
    } else if (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        // both inputs are constant - don't add a gate
        result.additive_constant = additive_constant + other.additive_constant;
    } else if (witness_index != IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        // one input is constant - don't add a gate, but update scaling factors
        result.additive_constant = additive_constant + other.additive_constant;
        result.multiplicative_constant = multiplicative_constant;
        result.witness_index = witness_index;
    } else if (witness_index == IS_CONSTANT && other.witness_index != IS_CONSTANT) {
        result.additive_constant = additive_constant + other.additive_constant;
        result.multiplicative_constant = other.multiplicative_constant;
        result.witness_index = other.witness_index;
    } else {
        barretenberg::fr T0;
        barretenberg::fr left = ctx->get_variable(witness_index);
        barretenberg::fr right = ctx->get_variable(other.witness_index);
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
    ASSERT(ctx || (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT));

    if (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        // Both inputs are constant - don't add a gate.
        // The value of a constant is tracked in `.additive_constant`.
        result.additive_constant = additive_constant * other.additive_constant;
    } else if (witness_index != IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        // One input is constant: don't add a gate, but update scaling factors.

        /**
         * Let:
         *   a := this;
         *   b := other;
         *   a.v := ctx->variables[this.witness_index];
         *   b.v := ctx->variables[other.witness_index];
         *   .mul = .multiplicative_constant
         *   .add = .additive_constant
         */

        /**
         * Value of this   = a.v * a.mul + a.add;
         * Value of other  = b.add
         * Value of result = a * b = a.v * [a.mul * b.add] + [a.add * b.add]
         *                             ^   ^result.mul       ^result.add
         *                             ^result.v
         */

        result.additive_constant = additive_constant * other.additive_constant;
        result.multiplicative_constant = multiplicative_constant * other.additive_constant;
        result.witness_index = witness_index;
    } else if (witness_index == IS_CONSTANT && other.witness_index != IS_CONSTANT) {
        // One input is constant: don't add a gate, but update scaling factors.

        /**
         * Value of this   = a.add;
         * Value of other  = b.v * b.mul + b.add
         * Value of result = a * b = b.v * [a.add * b.mul] + [a.add * b.add]
         *                             ^   ^result.mul       ^result.add
         *                             ^result.v
         */

        result.additive_constant = additive_constant * other.additive_constant;
        result.multiplicative_constant = other.multiplicative_constant * additive_constant;
        result.witness_index = other.witness_index;
    } else {
        // Both inputs map to circuit varaibles: create a `*` constraint.

        /**
         * Value of this   = a.v * a.mul + a.add;
         * Value of other  = b.v * b.mul + b.add;
         * Value of result = a * b
         *            = [a.v * b.v] * [a.mul * b.mul] + a.v * [a.mul * b.add] + b.v * [a.add * b.mul] + [a.ac * b.add]
         *            = [a.v * b.v] * [     q_m     ] + a.v * [     q_l     ] + b.v * [     q_r     ] + [    q_c     ]
         *            ^               ^Notice the add/mul_constants form selectors when a gate is created.
         *            |                Only the witnesses (pointed-to by the witness_indexes) form the wires in/out of
         *            |                the gate.
         *            ^This entire value is pushed to ctx->variables as a new witness. The
         *             implied additive & multiplicative constants of the new witness are 0 & 1 resp.
         * Left wire value: a.v
         * Right wire value: b.v
         * Output wire value: result.v (with q_o = -1)
         */

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
        const waffle::poly_triple gate_coefficients{ .a = witness_index,
                                                     .b = other.witness_index,
                                                     .c = result.witness_index,
                                                     .q_m = q_m,
                                                     .q_l = q_l,
                                                     .q_r = q_r,
                                                     .q_o = barretenberg::fr::neg_one(),
                                                     .q_c = q_c };
        ctx->create_poly_gate(gate_coefficients);
    }
    return result;
}

// Since in divide_no_zero_check, we check a/b=c by the constraint a=b*c, if a=b=0, we can set c to *any value*
// and it will pass the constraint. Hence, when not having prior knowledge of b not being zero it is essential to check.
template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::operator/(const field_t& other) const
{
    other.assert_is_not_zero("field_t::operator/ divisor is 0");
    return divide_no_zero_check(other);
}
template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::divide_no_zero_check(const field_t& other) const
{
    ComposerContext* ctx = (context == nullptr) ? other.context : context;
    field_t<ComposerContext> result(ctx);
    ASSERT(ctx || (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT));

    barretenberg::fr additive_multiplier = barretenberg::fr::one();

    if (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        // both inputs are constant - don't add a gate
        if (!(other.additive_constant == barretenberg::fr::zero())) {
            additive_multiplier = other.additive_constant.invert();
        }
        result.additive_constant = additive_constant * additive_multiplier;
    } else if (witness_index != IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        // one input is constant - don't add a gate, but update scaling factors
        if (!(other.additive_constant == barretenberg::fr::zero())) {
            additive_multiplier = other.additive_constant.invert();
        }
        result.additive_constant = additive_constant * additive_multiplier;
        result.multiplicative_constant = multiplicative_constant * additive_multiplier;
        result.witness_index = witness_index;
    } else if (witness_index == IS_CONSTANT && other.witness_index != IS_CONSTANT) {
        // numerator 0?
        if (get_value() == 0) {
            result.additive_constant = 0;
            result.multiplicative_constant = 1;
            result.witness_index = IS_CONSTANT;
        } else {
            barretenberg::fr q_m = other.multiplicative_constant;
            barretenberg::fr q_l = other.additive_constant;
            barretenberg::fr q_c = -get_value();
            barretenberg::fr out_value = get_value() / other.get_value();
            result.witness_index = ctx->add_variable(out_value);
            const waffle::poly_triple gate_coefficients{ .a = result.witness_index,
                                                         .b = other.witness_index,
                                                         .c = result.witness_index,
                                                         .q_m = q_m,
                                                         .q_l = q_l,
                                                         .q_r = 0,
                                                         .q_o = 0,
                                                         .q_c = q_c };
            ctx->create_poly_gate(gate_coefficients);
        }
    } else {
        // TODO SHOULD WE CARE ABOUT IF THE DIVISOR IS ZERO?
        barretenberg::fr left = ctx->get_variable(witness_index);
        barretenberg::fr right = ctx->get_variable(other.witness_index);
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

        T1 = T1.is_zero() ? 0 : T1.invert();
        out = T0 * T1;
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

        const waffle::poly_triple gate_coefficients{ .a = result.witness_index,
                                                     .b = other.witness_index,
                                                     .c = witness_index,
                                                     .q_m = q_m,
                                                     .q_l = q_l,
                                                     .q_r = q_r,
                                                     .q_o = q_o,
                                                     .q_c = q_c };
        ctx->create_poly_gate(gate_coefficients);
    }
    return result;
}
/**
 * @brief raise a field_t to a power of an exponent (field_t). Note that the exponent must not exceed 32 bits and is
 * implicitly range constrained.
 *
 * @returns this ** (exponent)
 */
template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::pow(const field_t& exponent) const
{
    auto* ctx = get_context() ? get_context() : exponent.get_context();

    bool exponent_constant = exponent.is_constant();

    uint256_t exponent_value = exponent.get_value();
    std::vector<bool_t<ComposerContext>> exponent_bits(32);
    for (size_t i = 0; i < exponent_bits.size(); ++i) {
        uint256_t value_bit = exponent_value & 1;
        bool_t<ComposerContext> bit;
        bit = exponent_constant ? bool_t<ComposerContext>(ctx, value_bit.data[0])
                                : witness_t<ComposerContext>(ctx, value_bit.data[0]);
        exponent_bits[31 - i] = (bit);
        exponent_value >>= 1;
    }

    if (!exponent_constant) {
        field_t<ComposerContext> exponent_accumulator(ctx, 0);
        for (const auto& bit : exponent_bits) {
            exponent_accumulator += exponent_accumulator;
            exponent_accumulator += bit;
        }
        exponent.assert_equal(exponent_accumulator, "field_t::pow exponent accumulator incorrect");
    }
    field_t accumulator(ctx, 1);
    field_t mul_coefficient = *this - 1;
    for (size_t i = 0; i < 32; ++i) {
        accumulator *= accumulator;
        const auto bit = exponent_bits[i];
        accumulator *= (mul_coefficient * bit + 1);
    }
    accumulator = accumulator.normalize();
    return accumulator;
}

/**
 * @returns `this * to_mul + to_add`
 */
template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::madd(const field_t& to_mul, const field_t& to_add) const
{
    ComposerContext* ctx =
        (context == nullptr) ? (to_mul.context == nullptr ? to_add.context : to_mul.context) : context;

    if ((to_mul.witness_index == IS_CONSTANT) && (to_add.witness_index == IS_CONSTANT) &&
        (witness_index == IS_CONSTANT)) {
        return ((*this) * to_mul + to_add);
    }

    // Let:
    //    a = this;
    //    b = to_mul;
    //    c = to_add;
    //    a.v = ctx->variables[this.witness_index];
    //    b.v = ctx->variables[to_mul.witness_index];
    //    c.v = ctx->variables[to_add.witness_index];
    //    .mul = .multiplicative_constant;
    //    .add = .additive_constant.
    //
    // result = a * b + c
    //   = (a.v * a.mul + a.add) * (b.v * b.mul + b.add) + (c.v * c.mul + c.add)
    //   = a.v * b.v * [a.mul * b.mul] + a.v * [a.mul * b.add] + b.v * [b.mul + a.add] + c.v * [c.mul] +
    //     [a.add * b.add + c.add]
    //   = a.v * b.v * [     q_m     ] + a.v * [     q_1     ] + b.v * [     q_2     ] + c.v * [ q_3 ] + [ q_c ]

    barretenberg::fr q_m = multiplicative_constant * to_mul.multiplicative_constant;
    barretenberg::fr q_1 = multiplicative_constant * to_mul.additive_constant;
    barretenberg::fr q_2 = to_mul.multiplicative_constant * additive_constant;
    barretenberg::fr q_3 = to_add.multiplicative_constant;
    barretenberg::fr q_c = additive_constant * to_mul.additive_constant + to_add.additive_constant;

    // Note: the value of a constant field_t is wholly tracked by the field_t's `additive_constant` member, which is
    // accounted for in the above-calculated selectors (`q_`'s). Therefore no witness (`variables[witness_index]`)
    // exists for constants, and so the field_t's corresponding wire value is set to `0` in the gate equation.
    barretenberg::fr a = witness_index == IS_CONSTANT ? barretenberg::fr(0) : ctx->get_variable(witness_index);
    barretenberg::fr b =
        to_mul.witness_index == IS_CONSTANT ? barretenberg::fr(0) : ctx->get_variable(to_mul.witness_index);
    barretenberg::fr c =
        to_add.witness_index == IS_CONSTANT ? barretenberg::fr(0) : ctx->get_variable(to_add.witness_index);

    barretenberg::fr out = a * b * q_m + a * q_1 + b * q_2 + c * q_3 + q_c;

    field_t<ComposerContext> result(ctx);
    result.witness_index = ctx->add_variable(out);

    const waffle::mul_quad gate_coefficients{
        .a = witness_index == IS_CONSTANT ? ctx->zero_idx : witness_index,
        .b = to_mul.witness_index == IS_CONSTANT ? ctx->zero_idx : to_mul.witness_index,
        .c = to_add.witness_index == IS_CONSTANT ? ctx->zero_idx : to_add.witness_index,
        .d = result.witness_index,
        .mul_scaling = q_m,
        .a_scaling = q_1,
        .b_scaling = q_2,
        .c_scaling = q_3,
        .d_scaling = -barretenberg::fr(1),
        .const_scaling = q_c,
    };
    ctx->create_big_mul_gate(gate_coefficients);
    return result;
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::add_two(const field_t& add_a, const field_t& add_b) const
{
    ComposerContext* ctx = (context == nullptr) ? (add_a.context == nullptr ? add_b.context : add_a.context) : context;

    if ((add_a.witness_index == IS_CONSTANT) && (add_b.witness_index == IS_CONSTANT) &&
        (witness_index == IS_CONSTANT)) {
        return ((*this) + add_a + add_b).normalize();
    }
    barretenberg::fr q_1 = multiplicative_constant;
    barretenberg::fr q_2 = add_a.multiplicative_constant;
    barretenberg::fr q_3 = add_b.multiplicative_constant;
    barretenberg::fr q_c = additive_constant + add_a.additive_constant + add_b.additive_constant;

    barretenberg::fr a = witness_index == IS_CONSTANT ? barretenberg::fr(0) : ctx->get_variable(witness_index);
    barretenberg::fr b =
        add_a.witness_index == IS_CONSTANT ? barretenberg::fr(0) : ctx->get_variable(add_a.witness_index);
    barretenberg::fr c =
        add_b.witness_index == IS_CONSTANT ? barretenberg::fr(0) : ctx->get_variable(add_b.witness_index);

    barretenberg::fr out = a * q_1 + b * q_2 + c * q_3 + q_c;

    field_t<ComposerContext> result(ctx);
    result.witness_index = ctx->add_variable(out);

    const waffle::mul_quad gate_coefficients{
        .a = witness_index == IS_CONSTANT ? ctx->zero_idx : witness_index,
        .b = add_a.witness_index == IS_CONSTANT ? ctx->zero_idx : add_a.witness_index,
        .c = add_b.witness_index == IS_CONSTANT ? ctx->zero_idx : add_b.witness_index,
        .d = result.witness_index,
        .mul_scaling = barretenberg::fr(0),
        .a_scaling = q_1,
        .b_scaling = q_2,
        .c_scaling = q_3,
        .d_scaling = -barretenberg::fr(1),
        .const_scaling = q_c,
    };
    ctx->create_big_mul_gate(gate_coefficients);
    return result;
}

template <typename ComposerContext> field_t<ComposerContext> field_t<ComposerContext>::normalize() const
{
    if (witness_index == IS_CONSTANT ||
        ((multiplicative_constant == barretenberg::fr::one()) && (additive_constant == barretenberg::fr::zero()))) {
        return *this;
    }

    // Value of this = this.v * this.mul + this.add; // where this.v = context->variables[this.witness_index]
    // Normalised result = result.v * 1 + 0;         // where result.v = this.v * this.mul + this.add
    // We need a new gate to enforce that the `result` was correctly calculated from `this`.

    field_t<ComposerContext> result(context);
    barretenberg::fr value = context->get_variable(witness_index);
    barretenberg::fr out;
    out = value * multiplicative_constant;
    out += additive_constant;

    result.witness_index = context->add_variable(out);
    result.additive_constant = barretenberg::fr::zero();
    result.multiplicative_constant = barretenberg::fr::one();

    // Aim of new gate: this.v * this.mul + this.add == result.v
    // <=>                           this.v * [this.mul] +                  result.v * [ -1] + [this.add] == 0
    // <=> this.v * this.v * [ 0 ] + this.v * [this.mul] + this.v * [ 0 ] + result.v * [ -1] + [this.add] == 0
    // <=> this.v * this.v * [q_m] + this.v * [   q_l  ] + this.v * [q_r] + result.v * [q_o] + [   q_c  ] == 0

    const waffle::add_triple gate_coefficients{ .a = witness_index,
                                                .b = witness_index,
                                                .c = result.witness_index,
                                                .a_scaling = multiplicative_constant,
                                                .b_scaling = 0,
                                                .c_scaling = barretenberg::fr::neg_one(),
                                                .const_scaling = additive_constant };

    context->create_add_gate(gate_coefficients);
    return result;
}

template <typename ComposerContext> void field_t<ComposerContext>::assert_is_zero(std::string const& msg) const
{
    if (get_value() != barretenberg::fr(0)) {
        context->failure(msg);
    }

    if (witness_index == IS_CONSTANT) {
        ASSERT(additive_constant == barretenberg::fr(0));
        return;
    }

    // Aim of new gate: this.v * this.mul + this.add == 0
    // I.e.:
    // this.v * 0 * [ 0 ] + this.v * [this.mul] + 0 * [ 0 ] + 0 * [ 0 ] + [this.add] == 0
    // this.v * 0 * [q_m] + this.v * [   q_l  ] + 0 * [q_r] + 0 * [q_o] + [   q_c  ] == 0

    ComposerContext* ctx = context;
    const waffle::poly_triple gate_coefficients{
        .a = witness_index,
        .b = ctx->zero_idx,
        .c = ctx->zero_idx,
        .q_m = barretenberg::fr(0),
        .q_l = multiplicative_constant,
        .q_r = barretenberg::fr(0),
        .q_o = barretenberg::fr(0),
        .q_c = additive_constant,
    };
    context->create_poly_gate(gate_coefficients);
}

template <typename ComposerContext> void field_t<ComposerContext>::assert_is_not_zero(std::string const& msg) const
{
    if (get_value() == barretenberg::fr(0)) {
        context->failure(msg);
        // We don't return; we continue with the function, for debugging purposes.
    }

    if (witness_index == IS_CONSTANT) {
        ASSERT(additive_constant != barretenberg::fr(0));
        return;
    }

    ComposerContext* ctx = context;
    if (get_value() == 0 && ctx) {
        ctx->failure(msg);
    }

    barretenberg::fr inverse_value = (get_value() == 0) ? 0 : get_value().invert();

    field_t<ComposerContext> inverse(witness_t<ComposerContext>(ctx, inverse_value));

    // Aim of new gate: `this` has an inverse (hence is not zero).
    // I.e.:
    //     (this.v * this.mul + this.add) * inverse.v == 1;
    // <=> this.v * inverse.v * [this.mul] + this.v * [ 0 ] + inverse.v * [this.add] + 0 * [ 0 ] + [ -1] == 0
    // <=> this.v * inverse.v * [   q_m  ] + this.v * [q_l] + inverse.v * [   q_r  ] + 0 * [q_o] + [q_c] == 0

    // (a * mul_const + add_const) * b - 1 = 0
    const waffle::poly_triple gate_coefficients{
        .a = witness_index,             // input value
        .b = inverse.witness_index,     // inverse
        .c = ctx->zero_idx,             // no output
        .q_m = multiplicative_constant, // a * b * mul_const
        .q_l = barretenberg::fr(0),     // a * 0
        .q_r = additive_constant,       // b * mul_const
        .q_o = barretenberg::fr(0),     // c * 0
        .q_c = barretenberg::fr(-1),    // -1
    };
    context->create_poly_gate(gate_coefficients);
}

template <typename ComposerContext> bool_t<ComposerContext> field_t<ComposerContext>::is_zero() const
{
    if (witness_index == IS_CONSTANT) {
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
    const waffle::poly_triple gate_coefficients_a{ .a = k.witness_index,
                                                   .b = k_inverse.witness_index,
                                                   .c = is_zero.witness_index,
                                                   .q_m = q_m,
                                                   .q_l = q_l,
                                                   .q_r = q_r,
                                                   .q_o = q_o,
                                                   .q_c = q_c };
    context->create_poly_gate(gate_coefficients_a);

    // is_zero * k_inverse - is_zero = 0
    q_o = barretenberg::fr::neg_one();
    q_c = barretenberg::fr::zero();
    const waffle::poly_triple gate_coefficients_b{ .a = is_zero.witness_index,
                                                   .b = k_inverse.witness_index,
                                                   .c = is_zero.witness_index,
                                                   .q_m = q_m,
                                                   .q_l = q_l,
                                                   .q_r = q_r,
                                                   .q_o = q_o,
                                                   .q_c = q_c };
    context->create_poly_gate(gate_coefficients_b);
    return is_zero;
}

template <typename ComposerContext> barretenberg::fr field_t<ComposerContext>::get_value() const
{
    if (witness_index != IS_CONSTANT) {
        ASSERT(context != nullptr);
        return (multiplicative_constant * context->get_variable(witness_index)) + additive_constant;
    } else {
        // A constant field_t's value is tracked wholly by its additive_constant member.
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
    field_t r(result);
    field_t x(witness_t(ctx, fc));

    const field_t& a = *this;
    const field_t& b = other;
    const field_t diff = a - b;

    const field_t t1 = r.madd(-x + 1, x);
    const field_t t2 = diff.madd(t1, r - 1);
    t2.assert_equal(0);

    return result;
}

template <typename ComposerContext>
bool_t<ComposerContext> field_t<ComposerContext>::operator!=(const field_t& other) const
{
    return !operator==(other);
}

template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::conditional_negate(const bool_t<ComposerContext>& predicate) const
{
    field_t<ComposerContext> predicate_field(predicate);
    field_t<ComposerContext> multiplicand = -(predicate_field + predicate_field);
    return multiplicand.madd(*this, *this);
}

// if predicate == true then return lhs, else return rhs
template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::conditional_assign(const bool_t<ComposerContext>& predicate,
                                                                      const field_t& lhs,
                                                                      const field_t& rhs)
{
    return (lhs - rhs).madd(predicate, rhs);
}

template <typename ComposerContext>
void field_t<ComposerContext>::create_range_constraint(const size_t num_bits, std::string const& msg) const
{
    if (num_bits == 0) {
        assert_is_zero("0-bit range_constraint on non-zero field_t.");
    } else {
        if (is_constant()) {
            ASSERT(uint256_t(get_value()).get_msb() < num_bits);
        } else {
            if constexpr (ComposerContext::type == waffle::ComposerType::PLOOKUP) {
                context->decompose_into_default_range(normalize().get_witness_index(),
                                                      num_bits,
                                                      waffle::UltraComposer::DEFAULT_PLOOKUP_RANGE_BITNUM,
                                                      msg);
            } else {
                context->decompose_into_base4_accumulators(normalize().get_witness_index(), num_bits, msg);
            }
        }
    }
}

/**
 * @brief Constrain that this field is equal to the given field.
 *
 * @warning: After calling this method, both field values *will* be equal, regardless of whether the constraint
 * succeeds or fails. This can lead to confusion when debugging. If you want to log the inputs, do so before
 * calling this method.
 */
template <typename ComposerContext>
void field_t<ComposerContext>::assert_equal(const field_t& rhs, std::string const& msg) const
{
    const field_t lhs = *this;
    ComposerContext* ctx = lhs.get_context() ? lhs.get_context() : rhs.get_context();

    if (lhs.is_constant() && rhs.is_constant()) {
        ASSERT(lhs.get_value() == rhs.get_value());
    } else if (lhs.is_constant()) {
        field_t right = rhs.normalize();
        ctx->assert_equal_constant(right.witness_index, lhs.get_value(), msg);
    } else if (rhs.is_constant()) {
        field_t left = lhs.normalize();
        ctx->assert_equal_constant(left.witness_index, rhs.get_value(), msg);
    } else {
        field_t left = lhs.normalize();
        field_t right = rhs.normalize();
        ctx->assert_equal(left.witness_index, right.witness_index, msg);
    }
}

template <typename ComposerContext>
void field_t<ComposerContext>::assert_not_equal(const field_t& rhs, std::string const& msg) const
{
    const field_t lhs = *this;
    const field_t diff = lhs - rhs;
    diff.assert_is_not_zero(msg);
}

template <typename ComposerContext>
void field_t<ComposerContext>::assert_is_in_set(const std::vector<field_t>& set, std::string const& msg) const
{
    const field_t input = *this;
    field_t product = (input - set[0]);
    for (size_t i = 1; i < set.size(); i++) {
        product *= (input - set[i]);
    }
    product.assert_is_zero(msg);
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

// Given T, stores the coefficients of the multilinear polynomial in t0,t1,t2, that on input a binary string b of
// length 3, equals T_b
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
    table[0] = T0;                                    // const coeff
    table[1] = T1 - T0;                               // t0 coeff
    table[2] = T2 - T0;                               // t1 coeff
    table[3] = T4 - T0;                               // t2 coeff
    table[4] = T3 - T2 - T1 + T0;                     // t0t1 coeff
    table[5] = T5 - T4 - T1 + T0;                     // t0t2 coeff
    table[6] = T6 - T4 - T2 + T0;                     // t1t2 coeff
    table[7] = T7 - T6 - T5 + T4 - T3 + T2 + T1 - T0; // t0t1t2 coeff
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

// we wish to compute the multilinear polynomial stored at point (t0,t1,t2) in a minimal number of gates.
// The straightforward thing would be eight multiplications to get the monomials and several additions between them
// It turns out you can do it in 7 multadd gates using the formula
// X:= ((t0*a012+a12)*t1+a2)*t2+a_const  - 3 gates
// Y:= (t0*a01+a1)*t1+X - 2 gates
// Z:= (t2*a02 + a0)*t0 + Y - 2 gates
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
void field_t<ComposerContext>::evaluate_linear_identity(const field_t& a,
                                                        const field_t& b,
                                                        const field_t& c,
                                                        const field_t& d)
{
    ComposerContext* ctx = a.context == nullptr
                               ? (b.context == nullptr ? (c.context == nullptr ? d.context : c.context) : b.context)
                               : a.context;

    if (a.witness_index == IS_CONSTANT && b.witness_index == IS_CONSTANT && c.witness_index == IS_CONSTANT &&
        d.witness_index == IS_CONSTANT) {
        return;
    }

    // validate that a + b + c + d = 0
    barretenberg::fr q_1 = a.multiplicative_constant;
    barretenberg::fr q_2 = b.multiplicative_constant;
    barretenberg::fr q_3 = c.multiplicative_constant;
    barretenberg::fr q_4 = d.multiplicative_constant;
    barretenberg::fr q_c = a.additive_constant + b.additive_constant + c.additive_constant + d.additive_constant;

    const waffle::add_quad gate_coefficients{
        a.witness_index == IS_CONSTANT ? ctx->zero_idx : a.witness_index,
        b.witness_index == IS_CONSTANT ? ctx->zero_idx : b.witness_index,
        c.witness_index == IS_CONSTANT ? ctx->zero_idx : c.witness_index,
        d.witness_index == IS_CONSTANT ? ctx->zero_idx : d.witness_index,
        q_1,
        q_2,
        q_3,
        q_4,
        q_c,
    };
    ctx->create_big_add_gate(gate_coefficients);
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

    if (a.witness_index == IS_CONSTANT && b.witness_index == IS_CONSTANT && c.witness_index == IS_CONSTANT &&
        d.witness_index == IS_CONSTANT) {
        return;
    }

    // validate that a * b + c + d = 0
    barretenberg::fr q_m = a.multiplicative_constant * b.multiplicative_constant;
    barretenberg::fr q_1 = a.multiplicative_constant * b.additive_constant;
    barretenberg::fr q_2 = b.multiplicative_constant * a.additive_constant;
    barretenberg::fr q_3 = c.multiplicative_constant;
    barretenberg::fr q_4 = d.multiplicative_constant;
    barretenberg::fr q_c = a.additive_constant * b.additive_constant + c.additive_constant + d.additive_constant;

    const waffle::mul_quad gate_coefficients{
        a.witness_index == IS_CONSTANT ? ctx->zero_idx : a.witness_index,
        b.witness_index == IS_CONSTANT ? ctx->zero_idx : b.witness_index,
        c.witness_index == IS_CONSTANT ? ctx->zero_idx : c.witness_index,
        d.witness_index == IS_CONSTANT ? ctx->zero_idx : d.witness_index,
        q_m,
        q_1,
        q_2,
        q_3,
        q_4,
        q_c,
    };
    ctx->create_big_mul_gate(gate_coefficients);
}

/**
 * Compute sum of inputs
 */
template <typename ComposerContext>
field_t<ComposerContext> field_t<ComposerContext>::accumulate(const std::vector<field_t>& input)
{
    if (input.size() == 0) {
        return field_t<ComposerContext>(nullptr, 0);
    }
    if (input.size() == 1) {
        return input[0]; //.normalize();
    }
    /**
     * If we are using UltraComposer, we can accumulate 3 values into a sum per gate.
     * We track a decumulating sum of values in the 4th wire of every row.
     * i.e. the 4th wire of the first row is the total output value
     *
     * At every gate, we subtract off three elements from `input`. Every gate apart from the final gate,
     * is an 'extended' addition gate, that includes the 4th wire of the next gate
     *
     * e.g. to accumulate 9 limbs, structure is:
     *
     * | l_1 | l_2 | l_3 | s_3 |
     * | l_4 | l_5 | l_6 | s_2 |
     * | l_7 | l_8 | l_9 | s_1 |
     *
     * We validate:
     *
     * s_3 - l_1 - l_2 - l_3 - s_2 = 0
     * s_2 - l_4 - l_5 - l_6 - s_1 = 0
     * s_1 - l_7 - l_8 - l_9 = 0
     *
     * If num elements is not a multiple of 3, the final gate will be padded with zero_idx wires
     **/
    if constexpr (ComposerContext::type == waffle::PLOOKUP) {
        ComposerContext* ctx = nullptr;
        std::vector<field_t> accumulator;
        field_t constant_term = 0;

        // Step 1: remove constant terms from input field elements
        for (const auto& element : input) {
            if (element.is_constant()) {
                constant_term += element;
            } else {
                accumulator.emplace_back(element);
            }
            ctx = (element.get_context() ? element.get_context() : ctx);
        }
        if (accumulator.size() == 0) {
            return constant_term;
        } else if (accumulator.size() != input.size()) {
            accumulator[0] += constant_term;
        }

        // Step 2: compute output value
        size_t num_elements = accumulator.size();
        barretenberg::fr output = 0;
        for (const auto& acc : accumulator) {
            output += acc.get_value();
        }

        // Step 3: pad accumulator to be a multiple of 3
        const size_t num_padding_wires = (num_elements % 3) == 0 ? 0 : 3 - (num_elements % 3);
        for (size_t i = 0; i < num_padding_wires; ++i) {
            accumulator.emplace_back(field_t<ComposerContext>::from_witness_index(ctx, ctx->zero_idx));
        }
        num_elements = accumulator.size();
        const size_t num_gates = (num_elements / 3);

        field_t total = witness_t(ctx, output);
        field_t accumulating_total = total;

        for (size_t i = 0; i < num_gates; ++i) {
            ctx->create_big_add_gate(
                {
                    accumulator[3 * i].get_witness_index(),
                    accumulator[3 * i + 1].get_witness_index(),
                    accumulator[3 * i + 2].get_witness_index(),
                    accumulating_total.witness_index,
                    accumulator[3 * i].multiplicative_constant,
                    accumulator[3 * i + 1].multiplicative_constant,
                    accumulator[3 * i + 2].multiplicative_constant,
                    -1,
                    accumulator[3 * i].additive_constant + accumulator[3 * i + 1].additive_constant +
                        accumulator[3 * i + 2].additive_constant,
                },
                ((i == num_gates - 1) ? false : true));
            barretenberg::fr new_total = accumulating_total.get_value() - accumulator[3 * i].get_value() -
                                         accumulator[3 * i + 1].get_value() - accumulator[3 * i + 2].get_value();
            accumulating_total = witness_t<ComposerContext>(ctx, new_total);
        }
        return total.normalize();
    } else if constexpr (ComposerContext::type == waffle::TURBO) {

        field_t total(0);
        bool odd_number = (input.size() & 0x01UL) == 0x01ULL;
        size_t end = input.size() - (odd_number ? 2 : 0);
        for (size_t i = 0; i < end; i += 2) {
            total = total.add_two(input[(size_t)i], input[(size_t)i + 1]);
        }
        if (odd_number) {
            total += input[input.size() - 1];
        }
        return total.normalize();
    }
    field_t<ComposerContext> total;
    for (const auto& item : input) {
        total += item;
    }
    return total;
}

template <typename ComposerContext>
std::array<field_t<ComposerContext>, 3> field_t<ComposerContext>::slice(const uint8_t msb, const uint8_t lsb) const
{
    ASSERT(msb >= lsb);
    ASSERT(msb < rollup::MAX_NO_WRAP_INTEGER_BIT_LENGTH); //  CODY: eek! Why is rollup info here? function input arg
                                                          //  msb_bound or something
    const field_t lhs = *this;
    ComposerContext* ctx = lhs.get_context();

    const uint256_t value = uint256_t(get_value());
    const auto msb_plus_one = uint32_t(msb) + 1;
    const auto hi_mask = ((uint256_t(1) << (256 - uint32_t(msb))) - 1);
    const auto hi = (value >> msb_plus_one) & hi_mask;

    const auto lo_mask = (uint256_t(1) << lsb) - 1;
    const auto lo = value & lo_mask;

    const auto slice_mask = ((uint256_t(1) << (uint32_t(msb - lsb) + 1)) - 1);
    const auto slice = (value >> lsb) & slice_mask;

    const field_t hi_wit = field_t(witness_t(ctx, hi));
    const field_t lo_wit = field_t(witness_t(ctx, lo));
    const field_t slice_wit = field_t(witness_t(ctx, slice));

    hi_wit.create_range_constraint(rollup::MAX_NO_WRAP_INTEGER_BIT_LENGTH - uint32_t(msb),
                                   "slice: hi value too large.");
    lo_wit.create_range_constraint(lsb, "slice: lo value too large.");
    slice_wit.create_range_constraint(msb_plus_one - lsb, "slice: sliced value too large.");
    assert_equal(
        ((hi_wit * field_t(uint256_t(1) << msb_plus_one)) + lo_wit + (slice_wit * field_t(uint256_t(1) << lsb))));

    std::array<field_t, 3> result = { lo_wit, slice_wit, hi_wit };
    return result;
}

/**
 * @brief Build a circuit allowing a user to prove that they have deomposed `this` into bits.
 *
 * @details A bit vector `result` is extracted and used to to construct a sum `sum` using the normal binary expansion.
 * Along the way, we extract a value `shifted_high_limb` which is equal to `sum_hi` in the natural decomposition
 *          `sum = sum_lo + 2**128*sum_hi`.
 * We impose a copy constraint between `sum` and `this` but that only imposes equality in `Fr`; it could be that
 * `result` has overflowed the modulus `r`. To impose a unique value of `result`, we constrain `sum` to satisfy `r - 1
 * >= sum >= 0`. In order to do this inside of `Fr`, we must reduce break the check down in the smaller checks so that
 * we can check non-negativity of integers using range constraints in Fr.
 *
 * At circuit compilation time we build the decomposition `r - 1 = p_lo + 2**128*p_hi`. Then desired schoolbook
 * subtraction is
 *                 p_hi - b       |        p_lo + b*2**128         (++foo++ is foo crossed out)
 *                 ++p_hi++       |           ++p_lo++               (b = 0, 1)
 *            -                   |
 *                  sum_hi        |             sum_lo
 *         -------------------------------------------------
 *     y_lo := p_hi - b - sum_hi  |  y_hi :=  p_lo + b*2**128 - sum_lo
 *
 * Here `b` is the boolean "a carry is necessary". Each of the resulting values can be checked for underflow by imposing
 * a small range constraint, since the negative of a small value in `Fr` is a large value in `Fr`.
 */
template <typename ComposerContext>
std::vector<bool_t<ComposerContext>> field_t<ComposerContext>::decompose_into_bits(
    const size_t num_bits,
    const std::function<witness_t<ComposerContext>(ComposerContext*, uint64_t, uint256_t)> get_bit) const
{
    ASSERT(num_bits <= 256);
    std::vector<bool_t<ComposerContext>> result(num_bits);

    const uint256_t val_u256 = static_cast<uint256_t>(get_value());
    field_t<ComposerContext> sum(context, 0);
    field_t<ComposerContext> shifted_high_limb(context, 0); // will equal high 128 bits, left shifted by 128 bits
    // TODO: Guido will make a PR that will fix an error here; hard-coded 127 is incorrect when 128 < num_bits < 256.
    // Extract bit vector and show that it has the same value as `this`.
    for (size_t i = 0; i < num_bits; ++i) {
        bool_t<ComposerContext> bit = get_bit(context, num_bits - 1 - i, val_u256);
        result[num_bits - 1 - i] = bit;
        barretenberg::fr scaling_factor_value = fr(2).pow(static_cast<uint64_t>(num_bits - 1 - i));
        field_t<ComposerContext> scaling_factor(context, scaling_factor_value);

        sum = sum + (scaling_factor * bit);
        if (i == 127)
            shifted_high_limb = sum;
    }

    this->assert_equal(sum); // `this` and `sum` are both normalized here.
    constexpr uint256_t modulus_minus_one = fr::modulus - 1;
    auto modulus_bits = modulus_minus_one.get_msb() + 1;
    // If value can be larger than modulus we must enforce unique representation
    if (num_bits >= modulus_bits) {
        // r - 1 = p_lo + 2**128 * p_hi
        const fr p_lo = modulus_minus_one.slice(0, 128);
        const fr p_hi = modulus_minus_one.slice(128, 256);

        // `shift` is used to shift high limbs. It has the dual purpose of representing a borrowed bit.
        const fr shift = fr(uint256_t(1) << 128);
        // We always borrow from 2**128*p_hi. We handle whether this was necessary later.
        // y_lo = (2**128 + p_lo) - sum_lo
        field_t<ComposerContext> y_lo = (-sum) + (p_lo + shift);
        y_lo += shifted_high_limb;
        y_lo.normalize();

        // A carry was necessary if and only if the 128th bit y_lo_hi of y_lo is 0.
        auto [y_lo_lo, y_lo_hi, zeros] = y_lo.slice(128, 128);
        // This copy constraint, along with the constraints of field_t::slice, imposes that y_lo has bit length 129.
        // Since the integer y_lo is at least -2**128+1, which has more than 129 bits in `Fr`, the implicit range
        // constraint shows that y_lo is non-negative.
        context->assert_equal(
            zeros.witness_index, context->zero_idx, "field_t: bit decomposition_fails: high limb non-zero");
        // y_borrow is the boolean "a carry was necessary"
        field_t<ComposerContext> y_borrow = -(y_lo_hi - 1);
        // If a carry was necessary, subtract that carry from p_hi
        // y_hi = (p_hi - y_borrow) - sum_hi
        field_t<ComposerContext> y_hi = -(shifted_high_limb / shift) + (p_hi);
        y_hi -= y_borrow;
        // As before, except that now the range constraint is explicit, this shows that y_hi is non-negative.
        y_hi.create_range_constraint(128, "field_t: bit decomposition fails: y_hi is too large.");
    }

    return result;
}

INSTANTIATE_STDLIB_TYPE(field_t);

} // namespace stdlib
} // namespace plonk
