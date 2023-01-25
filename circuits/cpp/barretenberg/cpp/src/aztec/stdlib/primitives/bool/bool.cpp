#include "bool.hpp"
#include "../composers/composers.hpp"
#include "honk/composer/standard_honk_composer.hpp"

using namespace barretenberg;
using namespace bonk;

namespace plonk {
namespace stdlib {

template <typename ComposerContext>
bool_t<ComposerContext>::bool_t(const bool value)
    : context(nullptr)
    , witness_bool(value)
    , witness_inverted(false)
    , witness_index(IS_CONSTANT)
{}

template <typename ComposerContext>
bool_t<ComposerContext>::bool_t(ComposerContext* parent_context)
    : context(parent_context)
{
    witness_bool = false;
    witness_inverted = false;
    witness_index = IS_CONSTANT;
}

template <typename ComposerContext>
bool_t<ComposerContext>::bool_t(const witness_t<ComposerContext>& value)
    : context(value.context)
{
    ASSERT((value.witness == barretenberg::fr::zero()) || (value.witness == barretenberg::fr::one()));
    witness_index = value.witness_index;
    context->create_bool_gate(witness_index);
    witness_bool = (value.witness == barretenberg::fr::one());
    witness_inverted = false;
}

template <typename ComposerContext>
bool_t<ComposerContext>::bool_t(ComposerContext* parent_context, const bool value)
    : context(parent_context)
{
    context = parent_context;
    witness_index = IS_CONSTANT;
    witness_bool = value;
    witness_inverted = false;
}

template <typename ComposerContext>
bool_t<ComposerContext>::bool_t(const bool_t<ComposerContext>& other)
    : context(other.context)
{
    witness_index = other.witness_index;
    witness_bool = other.witness_bool;
    witness_inverted = other.witness_inverted;
}

template <typename ComposerContext>
bool_t<ComposerContext>::bool_t(bool_t<ComposerContext>&& other)
    : context(other.context)
{
    witness_index = other.witness_index;
    witness_bool = other.witness_bool;
    witness_inverted = other.witness_inverted;
}

template <typename ComposerContext> bool_t<ComposerContext>& bool_t<ComposerContext>::operator=(const bool other)
{
    context = nullptr;
    witness_index = IS_CONSTANT;
    witness_bool = other;
    witness_inverted = false;
    return *this;
}

template <typename ComposerContext> bool_t<ComposerContext>& bool_t<ComposerContext>::operator=(const bool_t& other)
{
    context = other.context;
    witness_index = other.witness_index;
    witness_bool = other.witness_bool;
    witness_inverted = other.witness_inverted;
    return *this;
}

template <typename ComposerContext> bool_t<ComposerContext>& bool_t<ComposerContext>::operator=(bool_t&& other)
{
    context = other.context;
    witness_index = other.witness_index;
    witness_bool = other.witness_bool;
    witness_inverted = other.witness_inverted;
    return *this;
}

template <typename ComposerContext>
bool_t<ComposerContext>& bool_t<ComposerContext>::operator=(const witness_t<ComposerContext>& other)
{
    ASSERT((other.witness == barretenberg::fr::one()) || (other.witness == barretenberg::fr::zero()));
    context = other.context;
    witness_bool = (other.witness == barretenberg::fr::zero()) ? false : true;
    witness_index = other.witness_index;
    witness_inverted = false;
    context->create_bool_gate(witness_index);
    return *this;
}

template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::operator&(const bool_t& other) const
{
    bool_t<ComposerContext> result(context == nullptr ? other.context : context);
    bool left = witness_inverted ^ witness_bool;
    bool right = other.witness_inverted ^ other.witness_bool;

    ASSERT(result.context || (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT));
    if (witness_index != IS_CONSTANT && other.witness_index != IS_CONSTANT) {
        result.witness_bool = left & right;
        barretenberg::fr value = result.witness_bool ? barretenberg::fr::one() : barretenberg::fr::zero();
        result.witness_index = context->add_variable(value);
        result.witness_inverted = false;

        /**
         * A bool can be represented by a witness value `w` and an 'inverted' flag `i`
         *
         * A bool's value is defined via the equation:
         *      w + i - 2.i.w
         *
         * | w | i | w + i - 2.i.w |
         * | - | - | ------------- |
         * | 0 | 0 |       0       |
         * | 0 | 1 |       1       |
         * | 1 | 0 |       1       |
         * | 1 | 1 |       0       |
         *
         * For two bools (w_a, i_a), (w_b, i_b), the & operation is expressed as:
         *
         *   (w_a + i_a - 2.i_a.w_a).(w_b + i_b - 2.i_b.w_b)
         *
         * This can be rearranged to:
         *
         *      w_a.w_b.(1 - 2.i_b - 2.i_a + 4.i_a.i_b)     -> q_m coefficient
         *    + w_a.(i_b.(1 - 2.i_a))                       -> q_1 coefficient
         *    + w_b.(i_a.(1 - 2.i_b))                       -> q_2 coefficient
         *    + i_a.i_b                                     -> q_c coefficient
         *
         **/

        int i_a(witness_inverted);
        int i_b(other.witness_inverted);

        fr qm(1 - 2 * i_b - 2 * i_a + 4 * i_a * i_b);
        fr q1(i_b * (1 - 2 * i_a));
        fr q2(i_a * (1 - 2 * i_b));
        fr q3(-1);
        fr qc(i_a * i_b);

        const poly_triple gate_coefficients{
            witness_index, other.witness_index, result.witness_index, qm, q1, q2, q3, qc,
        };
        context->create_poly_gate(gate_coefficients);
    } else if (witness_index != IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        if (other.witness_bool ^ other.witness_inverted) {
            result = bool_t<ComposerContext>(*this);
        } else {
            result.witness_bool = false;
            result.witness_inverted = false;
            result.witness_index = IS_CONSTANT;
        }
    } else if (witness_index == IS_CONSTANT && other.witness_index != IS_CONSTANT) {
        if (witness_bool ^ witness_inverted) {
            result = bool_t<ComposerContext>(other);
        } else {
            result.witness_bool = false;
            result.witness_inverted = false;
            result.witness_index = IS_CONSTANT;
        }
    } else {
        result.witness_bool = left & right;
        result.witness_index = IS_CONSTANT;
        result.witness_inverted = false;
    }
    return result;
}

template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::operator|(const bool_t& other) const
{
    bool_t<ComposerContext> result(context == nullptr ? other.context : context);

    ASSERT(result.context || (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT));

    result.witness_bool = (witness_bool ^ witness_inverted) | (other.witness_bool ^ other.witness_inverted);
    barretenberg::fr value = result.witness_bool ? barretenberg::fr::one() : barretenberg::fr::zero();
    result.witness_inverted = false;
    if ((other.witness_index != IS_CONSTANT) && (witness_index != IS_CONSTANT)) {
        result.witness_index = context->add_variable(value);
        // result = A + B - AB, where A,B are the "real" values of the variables. But according to whether
        // witness_inverted flag is true, we need to invert the input. Hence, we look at four cases, and compute the
        // relevent coefficients of the selector q_1,q_2,q_m,q_c in each case
        barretenberg::fr multiplicative_coefficient;
        barretenberg::fr left_coefficient;
        barretenberg::fr right_coefficient;
        barretenberg::fr constant_coefficient;
        // a inverted: (1-a) + b - (1-a)b = 1-a+ab
        // ==> q_1=-1,q_2=0,q_m=1,q_c=1
        if (witness_inverted && !other.witness_inverted) {
            multiplicative_coefficient = barretenberg::fr::one();
            left_coefficient = barretenberg::fr::neg_one();
            right_coefficient = barretenberg::fr::zero();
            constant_coefficient = barretenberg::fr::one();
        }
        // b inverted: a + (1-b) - a(1-b) = 1-b+ab
        // ==> q_1=0,q_2=-1,q_m=1,q_c=1
        else if (!witness_inverted && other.witness_inverted) {
            multiplicative_coefficient = barretenberg::fr::one();
            left_coefficient = barretenberg::fr::zero();
            right_coefficient = barretenberg::fr::neg_one();
            constant_coefficient = barretenberg::fr::one();
        }
        // Both inverted: (1 - a) + (1 - b) - (1 - a)(1 - b) = 2 - a - b - (1 -a -b +ab) = 1 - ab
        // ==> q_m=-1,q_1=0,q_2=0,q_c=1
        else if (witness_inverted && other.witness_inverted) {
            multiplicative_coefficient = barretenberg::fr::neg_one();
            left_coefficient = barretenberg::fr::zero();
            right_coefficient = barretenberg::fr::zero();
            constant_coefficient = barretenberg::fr::one();
        }
        // No inversions: a + b - ab ==> q_m=-1,q_1=1,q_2=1,q_c=0
        else {
            multiplicative_coefficient = barretenberg::fr::neg_one();
            left_coefficient = barretenberg::fr::one();
            right_coefficient = barretenberg::fr::one();
            constant_coefficient = barretenberg::fr::zero();
        }
        const poly_triple gate_coefficients{
            witness_index,    other.witness_index, result.witness_index,        multiplicative_coefficient,
            left_coefficient, right_coefficient,   barretenberg::fr::neg_one(), constant_coefficient
        };
        context->create_poly_gate(gate_coefficients);
    } else if (witness_index != IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        if (other.witness_bool ^ other.witness_inverted) {
            result.witness_index = IS_CONSTANT;
            result.witness_bool = true;
            result.witness_inverted = false;
        } else {
            result = bool_t<ComposerContext>(*this);
        }
    } else if (witness_index == IS_CONSTANT && other.witness_index != IS_CONSTANT) {
        if (witness_bool ^ witness_inverted) {
            result.witness_index = IS_CONSTANT;
            result.witness_bool = true;
            result.witness_inverted = false;
        } else {
            result = bool_t<ComposerContext>(other);
        }
    } else {
        result.witness_inverted = false;
        result.witness_index = IS_CONSTANT;
    }
    return result;
}

template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::operator^(const bool_t& other) const
{
    bool_t<ComposerContext> result(context == nullptr ? other.context : context);

    ASSERT(result.context || (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT));

    result.witness_bool = (witness_bool ^ witness_inverted) ^ (other.witness_bool ^ other.witness_inverted);
    barretenberg::fr value = result.witness_bool ? barretenberg::fr::one() : barretenberg::fr::zero();
    result.witness_inverted = false;

    if ((other.witness_index != IS_CONSTANT) && (witness_index != IS_CONSTANT)) {
        result.witness_index = context->add_variable(value);
        // norm a, norm b: a + b - 2ab
        // inv  a, norm b: (1 - a) + b - 2(1 - a)b = 1 - a - b + 2ab
        // norm a, inv  b: a + (1 - b) - 2(a)(1 - b) = 1 - a - b + 2ab
        // inv  a, inv  b: (1 - a) + (1 - b) - 2(1 - a)(1 - b) = a + b - 2ab
        barretenberg::fr multiplicative_coefficient;
        barretenberg::fr left_coefficient;
        barretenberg::fr right_coefficient;
        barretenberg::fr constant_coefficient;
        if ((witness_inverted && other.witness_inverted) || (!witness_inverted && !other.witness_inverted)) {
            multiplicative_coefficient = (barretenberg::fr::neg_one() + barretenberg::fr::neg_one());
            left_coefficient = barretenberg::fr::one();
            right_coefficient = barretenberg::fr::one();
            constant_coefficient = barretenberg::fr::zero();
        } else {
            multiplicative_coefficient = barretenberg::fr::one() + barretenberg::fr::one();
            left_coefficient = barretenberg::fr::neg_one();
            right_coefficient = barretenberg::fr::neg_one();
            constant_coefficient = barretenberg::fr::one();
        }
        const poly_triple gate_coefficients{
            witness_index,    other.witness_index, result.witness_index,        multiplicative_coefficient,
            left_coefficient, right_coefficient,   barretenberg::fr::neg_one(), constant_coefficient
        };
        context->create_poly_gate(gate_coefficients);
    } else if (witness_index != IS_CONSTANT && other.witness_index == IS_CONSTANT) {
        // witness ^ 1 = !witness
        if (other.witness_bool ^ other.witness_inverted) {
            result = !bool_t<ComposerContext>(*this);
        } else {
            result = bool_t<ComposerContext>(*this);
        }
    } else if (witness_index == IS_CONSTANT && other.witness_index != IS_CONSTANT) {
        if (witness_bool ^ witness_inverted) {
            result = !bool_t<ComposerContext>(other);
        } else {
            result = bool_t<ComposerContext>(other);
        }
    } else {
        result.witness_inverted = false;
        result.witness_index = IS_CONSTANT;
    }
    return result;
}

template <typename ComposerContext> bool_t<ComposerContext> bool_t<ComposerContext>::operator!() const
{
    bool_t<ComposerContext> result(*this);
    result.witness_inverted = !result.witness_inverted;
    return result;
}

template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::operator==(const bool_t& other) const
{
    ASSERT(context || other.context || (witness_index == IS_CONSTANT && other.witness_index == IS_CONSTANT));
    if ((other.witness_index == IS_CONSTANT) && (witness_index == IS_CONSTANT)) {
        bool_t<ComposerContext> result(context == nullptr ? other.context : context);
        result.witness_bool = (witness_bool ^ witness_inverted) == (other.witness_bool ^ other.witness_inverted);
        result.witness_index = IS_CONSTANT;
        return result;
    } else if ((witness_index != IS_CONSTANT) && (other.witness_index == IS_CONSTANT)) {
        if (other.witness_bool ^ other.witness_inverted) {
            return (*this);
        } else {
            return !(*this);
        }
    } else if ((witness_index == IS_CONSTANT) && (other.witness_index != IS_CONSTANT)) {
        if (witness_bool ^ witness_inverted) {
            return other;
        } else {
            return !(other);
        }
    } else {
        bool_t<ComposerContext> result(context == nullptr ? other.context : context);
        result.witness_bool = (witness_bool ^ witness_inverted) == (other.witness_bool ^ other.witness_inverted);
        barretenberg::fr value = result.witness_bool ? barretenberg::fr::one() : barretenberg::fr::zero();
        result.witness_index = context->add_variable(value);
        // norm a, norm b or both inv: 1 - a - b + 2ab
        // inv a or inv b = a + b - 2ab
        barretenberg::fr multiplicative_coefficient;
        barretenberg::fr left_coefficient;
        barretenberg::fr right_coefficient;
        barretenberg::fr constant_coefficient;
        if ((witness_inverted && other.witness_inverted) || (!witness_inverted && !other.witness_inverted)) {
            multiplicative_coefficient = barretenberg::fr::one() + barretenberg::fr::one();
            left_coefficient = barretenberg::fr::neg_one();
            right_coefficient = barretenberg::fr::neg_one();
            constant_coefficient = barretenberg::fr::one();
        } else {
            multiplicative_coefficient = (barretenberg::fr::neg_one() + barretenberg::fr::neg_one());
            left_coefficient = barretenberg::fr::one();
            right_coefficient = barretenberg::fr::one();
            constant_coefficient = barretenberg::fr::zero();
        }
        const poly_triple gate_coefficients{
            witness_index,    other.witness_index, result.witness_index,        multiplicative_coefficient,
            left_coefficient, right_coefficient,   barretenberg::fr::neg_one(), constant_coefficient
        };
        context->create_poly_gate(gate_coefficients);
        return result;
    }
}

template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::operator!=(const bool_t<ComposerContext>& other) const
{
    return operator^(other);
}

template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::operator&&(const bool_t<ComposerContext>& other) const
{
    return operator&(other);
}

template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::operator||(const bool_t<ComposerContext>& other) const
{
    return operator|(other);
}

template <typename ComposerContext>
void bool_t<ComposerContext>::assert_equal(const bool_t& rhs, std::string const& msg) const
{
    const bool_t lhs = *this;
    ComposerContext* ctx = lhs.get_context() ? lhs.get_context() : rhs.get_context();

    if (lhs.is_constant() && rhs.is_constant()) {
        ASSERT(lhs.get_value() == rhs.get_value());
    } else if (lhs.is_constant()) {
        // if rhs is inverted, flip the value of the lhs constant
        bool lhs_value = rhs.witness_inverted ? !lhs.get_value() : lhs.get_value();
        ctx->assert_equal_constant(rhs.witness_index, lhs_value, msg);
    } else if (rhs.is_constant()) {
        // if lhs is inverted, flip the value of the rhs constant
        bool rhs_value = lhs.witness_inverted ? !rhs.get_value() : rhs.get_value();
        ctx->assert_equal_constant(lhs.witness_index, rhs_value, msg);
    } else {
        auto left = lhs;
        auto right = rhs;
        // we need to normalize iff lhs or rhs has an inverted witness (but not both)
        if (lhs.witness_inverted ^ rhs.witness_inverted) {
            left = left.normalize();
            right = right.normalize();
        }
        ctx->assert_equal(left.witness_index, right.witness_index, msg);
    }
}

template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::implies(const bool_t<ComposerContext>& other) const
{
    return (!(*this) || other); // P => Q is equiv. to !P || Q (not(P) or Q).
}

template <typename ComposerContext>
void bool_t<ComposerContext>::must_imply(const bool_t& other, std::string const& msg) const
{
    (this->implies(other)).assert_equal(true, msg);
}

// A "double-implication" (<=>),
// a.k.a "iff", a.k.a. "biconditional"
template <typename ComposerContext>
bool_t<ComposerContext> bool_t<ComposerContext>::implies_both_ways(const bool_t<ComposerContext>& other) const
{
    return (!(*this) ^ other); // P <=> Q is equiv. to !(P ^ Q) (not(P xor Q)).
}

template <typename ComposerContext> bool_t<ComposerContext> bool_t<ComposerContext>::normalize() const
{
    if (is_constant() || !witness_inverted) {
        return *this;
    }

    barretenberg::fr value = witness_bool ^ witness_inverted ? barretenberg::fr::one() : barretenberg::fr::zero();

    uint32_t new_witness = context->add_variable(value);
    uint32_t new_value = witness_bool ^ witness_inverted;

    barretenberg::fr q_l;
    barretenberg::fr q_c;

    q_l = witness_inverted ? barretenberg::fr::neg_one() : barretenberg::fr::one();
    q_c = witness_inverted ? barretenberg::fr::one() : barretenberg::fr::zero();

    barretenberg::fr q_o = barretenberg::fr::neg_one();
    barretenberg::fr q_m = barretenberg::fr::zero();
    barretenberg::fr q_r = barretenberg::fr::zero();

    const poly_triple gate_coefficients{ witness_index, witness_index, new_witness, q_m, q_l, q_r, q_o, q_c };

    context->create_poly_gate(gate_coefficients);

    witness_index = new_witness;
    witness_bool = new_value;
    witness_inverted = false;
    return *this;
}

INSTANTIATE_STDLIB_TYPE(bool_t);

} // namespace stdlib
} // namespace plonk
