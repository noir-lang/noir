#include "uint.hpp"
#include "../composers/composers.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const witness_t<Composer>& witness)
    : context(witness.context)
    , additive_constant(0)
    , witness_status(WitnessStatus::OK)
    , accumulators(context->create_range_constraint(witness.witness_index, width))
    , witness_index(accumulators[(width >> 1) - 1])
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const field_t<Composer>& value)
    : context(value.context)
    , additive_constant(0)
    , witness_status(WitnessStatus::OK)
    , accumulators()
    , witness_index(UINT32_MAX)
{
    if (value.witness_index == UINT32_MAX) {
        additive_constant = value.additive_constant;
    } else {
        field_t<Composer> norm = value.normalize();
        accumulators = context->create_range_constraint(norm.witness_index, width);
        witness_index = accumulators[(width >> 1) - 1];
    }
}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(Composer* composer, const uint256_t& value)
    : context(composer)
    , additive_constant(value)
    , witness_status(WitnessStatus::OK)
    , accumulators()
    , witness_index(UINT32_MAX)
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const uint256_t& value)
    : context(nullptr)
    , additive_constant(value)
    , witness_status(WitnessStatus::OK)
    , accumulators()
    , witness_index(UINT32_MAX)
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const byte_array<Composer>& other)
    : context(other.get_context())
    , additive_constant(0)
    , witness_status(WitnessStatus::WEAK_NORMALIZED)
    , accumulators()
    , witness_index(UINT32_MAX)
{
    field_t<Composer> accumulator(context, fr::zero());
    field_t<Composer> scaling_factor(context, fr::one());
    for (size_t i = 0; i < other.bits().size(); ++i) {
        accumulator = accumulator + scaling_factor * other.get_bit(i);
        scaling_factor = scaling_factor + scaling_factor;
    }
    if (accumulator.witness_index == UINT32_MAX) {
        additive_constant = uint256_t(accumulator.additive_constant);
    } else {
        witness_index = accumulator.witness_index;
    }
}

template <typename Composer, typename Native>
 uint<Composer, Native>::uint(Composer* parent_context, const std::array<bool_t<Composer>, width>& wires)
    : uint<Composer, Native>(parent_context, std::vector<bool_t<Composer>>(wires.begin(), wires.end()))
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(Composer* parent_context, const std::vector<bool_t<Composer>>& wires)
    : context(parent_context)
    , additive_constant(0)
    , witness_status(WitnessStatus::WEAK_NORMALIZED)
    , accumulators()
    , witness_index(UINT32_MAX)
{
    field_t<Composer> accumulator(context, fr::zero());
    field_t<Composer> scaling_factor(context, fr::one());
    for (size_t i = 0; i < wires.size(); ++i) {
        accumulator = accumulator + scaling_factor * wires[i];
        scaling_factor = scaling_factor + scaling_factor;
    }
    if (accumulator.witness_index == UINT32_MAX) {
        additive_constant = uint256_t(accumulator.additive_constant);
    } else {
        witness_index = accumulator.witness_index;
    }
}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const uint& other)
    : context(other.context)
    , additive_constant(other.additive_constant)
    , witness_status(other.witness_status)
    , accumulators(other.accumulators)
    , witness_index(other.witness_index)
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(uint&& other)
    : context(other.context)
    , additive_constant(other.additive_constant)
    , witness_status(other.witness_status)
    , accumulators(other.accumulators)
    , witness_index(other.witness_index)
{}

template <typename Composer, typename Native>
uint<Composer, Native>& uint<Composer, Native>::operator=(const uint& other)
{
    context = other.context;
    additive_constant = other.additive_constant;
    witness_status = other.witness_status;
    accumulators = other.accumulators;
    witness_index = other.witness_index;
    return *this;
}

template <typename Composer, typename Native> uint<Composer, Native>& uint<Composer, Native>::operator=(uint&& other)
{
    context = other.context;
    additive_constant = other.additive_constant;
    witness_status = other.witness_status;
    accumulators = other.accumulators;
    witness_index = other.witness_index;
    return *this;
}

template <typename Context, typename Native> uint<Context, Native>::operator byte_array<Context>() const
{
    if (is_constant()) {
        std::vector<bool_t<Context>> bits;
        bits.reserve(width);
        for (size_t i = width - 1; i < width; --i) {
            bool_t<Context> bit(context, additive_constant.get_bit(i));
            bits.emplace_back(bit);
        }
        return byte_array(context, bits);
    }

    if (witness_status == WitnessStatus::NOT_NORMALIZED) {
        weak_normalize();
    }
    // TODO: we should create a native type that works with packed data or quads, to
    // take advantage of our range constraint

    std::vector<bool_t<Context>> bits;
    bits.resize(width);
    uint256_t target = get_value();
    uint256_t accumulator = 0;
    uint256_t scale_factor = 1;
    uint32_t accumulator_idx = context->zero_idx;
    for (size_t i = 0; i < width; i += 2) {
        bool lo_val = target.get_bit(i);
        bool hi_val = target.get_bit(i + 1);
        bool_t lo = witness_t(context, lo_val);
        bool_t hi = witness_t(context, hi_val);

        uint256_t next_accumulator =
            accumulator + (lo_val ? scale_factor : 0) + (hi_val ? scale_factor + scale_factor : 0);
        waffle::add_quad gate{ lo.witness_index, hi.witness_index,
                               accumulator_idx,  context->add_variable(next_accumulator),
                               scale_factor,     scale_factor + scale_factor,
                               fr::one(),        fr::neg_one(),
                               fr::zero() };

        context->create_big_add_gate(gate);

        accumulator = next_accumulator;
        accumulator_idx = gate.d;
        scale_factor = scale_factor + scale_factor;
        scale_factor = scale_factor + scale_factor;

        bits[width - 1 - i] = lo;
        bits[width - 2 - i] = hi;
    }

    return byte_array(context, bits);
}

template <typename Context, typename Native> uint<Context, Native>::operator field_t<Context>() const
{
    normalize();
    field_t<Context> target(context);
    target.witness_index = witness_index;
    target.additive_constant = is_constant() ? fr(additive_constant) : fr::zero();
    return target;
}

template <typename Composer, typename Native> uint<Composer, Native> uint<Composer, Native>::weak_normalize() const
{
    if (!context || is_constant()) {
        return *this;
    }
    if (witness_status == WitnessStatus::WEAK_NORMALIZED) {
        return *this;
    }
    if (witness_status == WitnessStatus::NOT_NORMALIZED) {
        const uint256_t value = get_unbounded_value();
        const uint256_t overflow = value >> width;
        const uint256_t remainder = value & MASK;
        const waffle::add_quad gate{
            witness_index,
            context->zero_idx,
            context->add_variable(remainder),
            context->add_variable(overflow),
            fr::one(),
            fr::zero(),
            fr::neg_one(),
            -fr(CIRCUIT_UINT_MAX_PLUS_ONE),
            (additive_constant & MASK),
        };

        context->create_balanced_add_gate(gate);

        witness_index = gate.c;
        witness_status = WitnessStatus::WEAK_NORMALIZED;
        additive_constant = 0;
    }
    return *this;
}

template <typename Composer, typename Native> uint<Composer, Native> uint<Composer, Native>::normalize() const
{
    if (!context || is_constant()) {
        return *this;
    }

    if (witness_status == WitnessStatus::WEAK_NORMALIZED) {
        accumulators = context->create_range_constraint(witness_index, width);
        witness_index = accumulators[(width >> 1) - 1];
        witness_status = WitnessStatus::OK;
    }
    if (witness_status == WitnessStatus::NOT_NORMALIZED) {
        weak_normalize();
        accumulators = context->create_range_constraint(witness_index, width);
        witness_index = accumulators[(width >> 1) - 1];
        witness_status = WitnessStatus::OK;
    }
    return *this;
}

template <typename Composer, typename Native> uint256_t uint<Composer, Native>::get_value() const
{
    if (!context || is_constant()) {
        return additive_constant;
    }
    return (uint256_t(context->get_variable(witness_index)) + additive_constant) & MASK;
}

template <typename Composer, typename Native> uint256_t uint<Composer, Native>::get_unbounded_value() const
{
    if (!context || is_constant()) {
        return additive_constant;
    }
    return (uint256_t(context->get_variable(witness_index)) + additive_constant);
}

template <typename Composer, typename Native> bool_t<Composer> uint<Composer, Native>::at(const size_t bit_index) const
{
    if (is_constant()) {
        return bool_t<Composer>(context, get_value().get_bit(bit_index));
    }
    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    const size_t pivot_index = ((width >> 1) - (bit_index >> 1UL)) - 1;
    uint32_t left_idx = (pivot_index == 0) ? context->zero_idx : accumulators[pivot_index - 1];
    uint32_t right_idx = accumulators[pivot_index];
    uint256_t quad =
        uint256_t(context->get_variable(right_idx)) - uint256_t(context->get_variable(left_idx)) * uint256_t(4);

    // if 'index' is odd, we want a low bit
    if ((bit_index & 1UL) == 0UL) {
        // we want a low bit
        uint256_t lo_bit = quad & 1;
        waffle::add_quad gate{
            context->add_variable(lo_bit), // our extracted bit
            context->zero_idx,             // no explicit need to add high bit - extract gate does that for us
            right_idx,                     // large accumulator
            left_idx,                      // small accumulator
            fr(3),                         // 3 * lo_bit + 6 * hi_bit = 3 * a[pivot] - 12 * a[pivot - 1]
            fr::zero(),                    // 0
            -fr(3),                        // -3 * a[pivot]
            fr(12),                        // 12 * a[pivot - 1]
            fr::zero()                     // 0
        };
        context->create_big_add_gate_with_bit_extraction(gate);
        bool_t<Composer> result;
        result.witness_index = gate.a;
        result.witness_bool = (lo_bit == 1) ? true : false;
        return result;
    }

    // if 'index' is even, we want a high bit
    uint256_t hi_bit = quad >> 1;

    waffle::add_quad gate{
        context->zero_idx,             // no need for the low bit
        context->add_variable(hi_bit), // our extracted bit
        right_idx,                     // large accumlator
        left_idx,                      // small accumulator
        fr::zero(),                    // 0
        -fr(6),                        // extracted bit is scaled by 6, so apply -6 to our high bit
        fr::zero(),                    // 0
        fr::zero(),                    // 0
        fr::zero()                     // 0
    };
    context->create_big_add_gate_with_bit_extraction(gate);
    bool_t<Composer> result;
    result.witness_index = gate.b;
    result.witness_bool = (hi_bit == 1) ? true : false;
    return result;
}

INSTANTIATE_STDLIB_TYPE_VA(uint, uint8_t);
INSTANTIATE_STDLIB_TYPE_VA(uint, uint16_t);
INSTANTIATE_STDLIB_TYPE_VA(uint, uint32_t);
INSTANTIATE_STDLIB_TYPE_VA(uint, uint64_t);

} // namespace stdlib
} // namespace plonk