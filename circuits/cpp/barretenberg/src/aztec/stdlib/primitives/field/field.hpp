#pragma once
#include "../composers/composers_fwd.hpp"
#include "../witness/witness.hpp"
#include <common/assert.hpp>

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class bool_t;

template <typename ComposerContext> class field_t {
  public:
    field_t(ComposerContext* parent_context = nullptr);
    field_t(ComposerContext* parent_context, const barretenberg::fr& value);

    field_t(const int value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = IS_CONSTANT;
    }

    field_t(const unsigned long long value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = IS_CONSTANT;
    }

    field_t(const unsigned int value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = IS_CONSTANT;
    }

    field_t(const unsigned long value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = IS_CONSTANT;
    }

    field_t(uint256_t const& value)
        : context(nullptr)
    {
        additive_constant = barretenberg::fr(value);
        multiplicative_constant = barretenberg::fr(0);
        witness_index = IS_CONSTANT;
    }

    field_t(const barretenberg::fr& value)
        : context(nullptr)
        , additive_constant(value)
        , multiplicative_constant(barretenberg::fr(1))
        , witness_index(IS_CONSTANT)
    {}

    field_t(const witness_t<ComposerContext>& value);

    field_t(const field_t& other)
        : context(other.context)
        , additive_constant(other.additive_constant)
        , multiplicative_constant(other.multiplicative_constant)
        , witness_index(other.witness_index)
    {}

    field_t(field_t&& other)
        : context(other.context)
        , additive_constant(other.additive_constant)
        , multiplicative_constant(other.multiplicative_constant)
        , witness_index(other.witness_index)
    {}

    field_t(const bool_t<ComposerContext>& other);

    static constexpr bool is_composite = false;
    static constexpr uint256_t modulus = barretenberg::fr::modulus;

    static field_t from_witness_index(ComposerContext* parent_context, const uint32_t witness_index);

    explicit operator bool_t<ComposerContext>();

    field_t& operator=(const field_t& other)
    {
        additive_constant = other.additive_constant;
        multiplicative_constant = other.multiplicative_constant;
        witness_index = other.witness_index;
        context = (other.context == nullptr ? nullptr : other.context);
        return *this;
    }

    field_t& operator=(field_t&& other)
    {
        additive_constant = other.additive_constant;
        multiplicative_constant = other.multiplicative_constant;
        witness_index = other.witness_index;
        context = (other.context == nullptr ? nullptr : other.context);
        return *this;
    }

    field_t operator+(const field_t& other) const;
    field_t operator-(const field_t& other) const;
    field_t operator*(const field_t& other) const;
    field_t operator/(const field_t& other) const;

    field_t sqr() const { return operator*(*this); }

    field_t operator+=(const field_t& other)
    {
        *this = *this + other;
        return *this;
    }
    field_t operator-=(const field_t& other)
    {
        *this = *this - other;
        return *this;
    }
    field_t operator*=(const field_t& other)
    {
        *this = *this * other;
        return *this;
    }
    field_t operator/=(const field_t& other)
    {
        *this = *this / other;
        return *this;
    }

    field_t invert() const { return (field_t(1) / field_t(*this)).normalize(); }

    static field_t coset_generator(const size_t generator_idx)
    {
        return field_t(barretenberg::fr::coset_generator(generator_idx));
    }

    static field_t external_coset_generator() { return field_t(barretenberg::fr::external_coset_generator()); }

    field_t operator-() const
    {
        field_t result(*this);
        result.multiplicative_constant = -multiplicative_constant;
        result.additive_constant = -additive_constant;

        return result;
    }

    field_t conditional_negate(const bool_t<ComposerContext>& predicate) const;

    void assert_equal(const field_t& rhs, std::string const& msg = "field_t::assert_equal") const
    {
        const field_t lhs = *this;
        ComposerContext* ctx = lhs.get_context() ? lhs.get_context() : rhs.get_context();

        if (lhs.witness_index == UINT32_MAX && rhs.witness_index == UINT32_MAX) {
            ASSERT(lhs.get_value() == rhs.get_value());
        } else if (lhs.witness_index == UINT32_MAX) {
            field_t right = rhs.normalize();
            ctx->assert_equal_constant(right.witness_index, lhs.get_value(), msg);
        } else if (rhs.witness_index == UINT32_MAX) {
            field_t left = lhs.normalize();
            ctx->assert_equal_constant(left.witness_index, rhs.get_value(), msg);
        } else {
            field_t left = lhs.normalize();
            field_t right = rhs.normalize();
            ctx->assert_equal(left.witness_index, right.witness_index, msg);
        }
    }

    static std::array<field_t, 4> preprocess_two_bit_table(const field_t& T0,
                                                           const field_t& T1,
                                                           const field_t& T2,
                                                           const field_t& T3);
    static field_t select_from_two_bit_table(const std::array<field_t, 4>& table,
                                             const bool_t<ComposerContext>& t1,
                                             const bool_t<ComposerContext>& t0);

    static std::array<field_t, 8> preprocess_three_bit_table(const field_t& T0,
                                                             const field_t& T1,
                                                             const field_t& T2,
                                                             const field_t& T3,
                                                             const field_t& T4,
                                                             const field_t& T5,
                                                             const field_t& T6,
                                                             const field_t& T7);
    static field_t select_from_three_bit_table(const std::array<field_t, 8>& table,
                                               const bool_t<ComposerContext>& t2,
                                               const bool_t<ComposerContext>& t1,
                                               const bool_t<ComposerContext>& t0);

    static void evaluate_polynomial_identity(const field_t& a, const field_t& b, const field_t& c, const field_t& d);

    /**
     * multiply *this by `to_mul` and add `to_add`
     * One `madd` call costs 1 constraint for Turbo plonk and Ultra plonk
     * */
    field_t madd(const field_t& to_mul, const field_t& to_add) const;

    // add_two costs 1 constraint for turbo/ultra plonk
    field_t add_two(const field_t& add_a, const field_t& add_b) const;
    bool_t<ComposerContext> operator==(const field_t& other) const;
    bool_t<ComposerContext> operator!=(const field_t& other) const;

    /**
     * normalize returns a field_t element where `multiplicative_constant = 1` and `additive_constant = 0`
     * i.e. the value is defined entirely by the composer variable that `witness_index` points to
     * If the witness_index is ever needed, `normalize` should be called first
     *
     * Will cost 1 constraint if the field element is not already normalized (or is constant)
     **/
    field_t normalize() const;

    barretenberg::fr get_value() const;

    ComposerContext* get_context() const { return context; }

    /**
     * is_zero will return a bool_t, and add constraints that enforce its correctness
     * N.B. If you want to ENFORCE that a field_t object is zero, use `assert_is_zero`
     **/
    bool_t<ComposerContext> is_zero() const;

    void assert_is_not_zero(std::string const& msg = "field_t::assert_is_not_zero") const;
    void assert_is_zero(std::string const& msg = "field_t::assert_is_zero") const;
    bool is_constant() const { return witness_index == IS_CONSTANT; }

    uint32_t get_witness_index() const { return witness_index; }

    mutable ComposerContext* context = nullptr;

    /**
     * additive_constant, multiplicative_constant are constant scaling factors applied to a field_t object.
     * We track these scaling factors, because we can apply the same scaling factors to Plonk wires when creating gates.
     * i.e. if we want to multiply a wire by a constant, or add a constant, we do not need to add extra gates to do
     *this. Instead, we track the scaling factors, and apply them to the relevant wires when adding constraints
     *
     * This also makes constant field_t objects effectively free. Where 'constant' is a circuit constant, not a C++
     *constant! e.g. the following 3 lines of code add 0 constraints into a circuit:
     *
     * field_t foo = 1;
     * field_t bar = 5;
     * field_t bar *= foo;
     *
     * Similarly if we add in:
     *
     * field_t zip = witness_t(context, 10);
     * zip *= bar + foo;
     *
     * The above adds 0 constraints, the only effect is that `zip`'s scaling factors have been modified. However if we
     *now add:
     *
     * field_t zap = witness_t(context, 50);
     * zip *= zap;
     *
     * This will add a constraint, as both zip and zap map to circuit witnesses
     **/
    mutable barretenberg::fr additive_constant;
    mutable barretenberg::fr multiplicative_constant;

    /**
     * Every composer object contains a list of 'witnesses', circuit variables that can be assigned to wires when
     *creating constraints `witness_index` describes a location in this container. i.e. it 'points' to a circuit
     *variable
     *
     * A witness is not the same thing as a 'wire' in a circuit. Multiple wires can be assigned to the same witness via
     *Plonk's copy constraints. Alternatively, a witness might not be assigned to any wires! This case would be similar
     *to an unused variable in a regular program
     *
     * e.g. if we write `field_t foo = witness_t(context, 100)`, this will add the value `100` into `context`'s list of
     *circuit variables. However if we do not use `foo` in any operations, then this value will never be assigned to a
     *wire in a circuit
     *
     * For a more in depth example, consider the following code:
     *
     * field_t foo = witness_t(context, 10);
     * field_t bar = witness_t(context, 50);
     * field_t baz = foo * (bar + 7);
     *
     * This will add 3 new circuit witnesses (10, 50, 570). One constraint will also be created, that validates `baz`
     *has been correctly constructed The composer will assign `foo, bar, baz` to wires `w_1, w_2, w_3` in a gate, and
     *check that:
     *
     *      w_1 * w_2 + w_1 * 7 - w_3 = 0
     *
     * If any of `foo, bar, baz` are used in future arithmetic, copy constraints will be automatically applied,
     * this ensure that all gate wires that map to `foo`, for example, will contain the same value
     *
     * If witness_index == IS_CONSTANT, the object represents a constant value.
     * i.e. a value that's hardcoded in the circuit, that a prover cannot change by modifying their witness transcript
     *
     * A Plonk gate is a mix of witness values and selector values. e.g. the regular PLONK arithmetic gate checks that:
     *
     *      w_1 * w_2 * q_m + w_1 * q_1 + w_2 * w_2 + w_3 * q_3 + q_c = 0
     *
     * The `w` value are wires, the `q` values are selector constants. If a field object contains a witness index, it
     *will be assigned to `w` values when constraints are applied. If it's a circuit constant, it will be assigned to
     *`q` values
     *
     * TLDR: witness_index is a pseudo pointer to a circuit witness
     **/
    mutable uint32_t witness_index = IS_CONSTANT;
};

template <typename ComposerContext> inline std::ostream& operator<<(std::ostream& os, field_t<ComposerContext> const& v)
{
    return os << v.get_value();
}

EXTERN_STDLIB_TYPE(field_t);

} // namespace stdlib
} // namespace plonk
