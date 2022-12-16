#include "uint.hpp"
#include "../../composers/composers.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename Composer, typename Native>
std::vector<uint32_t> uint_plookup<Composer, Native>::constrain_accumulators(Composer* context,
                                                                             const uint32_t witness_index) const
{
    const auto res = context->decompose_into_default_range(witness_index, width, bits_per_limb);
    return res;
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(const witness_t<Composer>& witness)
    : context(witness.context)
    , additive_constant(0)
    , witness_status(WitnessStatus::OK)
    , accumulators(constrain_accumulators(context, witness.witness_index))
    , witness_index(witness.witness_index)
{}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(const field_t<Composer>& value)
    : context(value.context)
    , additive_constant(0)
    , witness_status(WitnessStatus::OK)
    , accumulators()
    , witness_index(IS_CONSTANT)
{
    if (value.witness_index == IS_CONSTANT) {
        additive_constant = value.additive_constant;
    } else {
        field_t<Composer> norm = value.normalize();
        accumulators = constrain_accumulators(context, norm.get_witness_index());
        witness_index = norm.get_witness_index();
    }
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(Composer* composer, const uint256_t& value)
    : context(composer)
    , additive_constant(value)
    , witness_status(WitnessStatus::OK)
    , accumulators()
    , witness_index(IS_CONSTANT)
{}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(const uint256_t& value)
    : context(nullptr)
    , additive_constant(value)
    , witness_status(WitnessStatus::OK)
    , accumulators()
    , witness_index(IS_CONSTANT)
{}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(const byte_array<Composer>& other)
    : context(other.get_context())
    , additive_constant(0)
    , witness_status(WitnessStatus::WEAK_NORMALIZED)
    , accumulators()
    , witness_index(IS_CONSTANT)
{
    field_t<Composer> accumulator(context, fr::zero());
    field_t<Composer> scaling_factor(context, fr::one());
    const auto bytes = other.bytes();

    // TODO JUMP IN STEPS OF TWO
    for (size_t i = 0; i < bytes.size(); ++i) {
        accumulator = accumulator + scaling_factor * bytes[bytes.size() - 1 - i];
        scaling_factor = scaling_factor * fr(256);
    }
    accumulator = accumulator.normalize();
    if (accumulator.witness_index == IS_CONSTANT) {
        additive_constant = uint256_t(accumulator.additive_constant);
    } else {
        witness_index = accumulator.witness_index;
    }
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(Composer* parent_context, const std::array<bool_t<Composer>, width>& wires)
    : uint_plookup<Composer, Native>(parent_context, std::vector<bool_t<Composer>>(wires.begin(), wires.end()))
{}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(Composer* parent_context, const std::vector<bool_t<Composer>>& wires)
    : context(parent_context)
    , additive_constant(0)
    , witness_status(WitnessStatus::WEAK_NORMALIZED)
    , accumulators()
    , witness_index(IS_CONSTANT)
{
    field_t<Composer> accumulator(context, fr::zero());
    field_t<Composer> scaling_factor(context, fr::one());

    // TODO JUMP IN STEPS OF TWO
    for (size_t i = 0; i < wires.size(); ++i) {
        accumulator = accumulator + scaling_factor * field_t<Composer>(wires[i]);
        scaling_factor = scaling_factor + scaling_factor;
    }
    accumulator = accumulator.normalize();
    if (accumulator.witness_index == IS_CONSTANT) {
        additive_constant = uint256_t(accumulator.additive_constant);
    } else {
        witness_index = accumulator.witness_index;
    }
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(const uint_plookup& other)
    : context(other.context)
    , additive_constant(other.additive_constant)
    , witness_status(other.witness_status)
    , accumulators(other.accumulators)
    , witness_index(other.witness_index)
{}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>::uint_plookup(uint_plookup&& other)
    : context(other.context)
    , additive_constant(other.additive_constant)
    , witness_status(other.witness_status)
    , accumulators(other.accumulators)
    , witness_index(other.witness_index)
{}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>& uint_plookup<Composer, Native>::operator=(const uint_plookup& other)
{
    context = other.context;
    additive_constant = other.additive_constant;
    witness_status = other.witness_status;
    accumulators = other.accumulators;
    witness_index = other.witness_index;
    return *this;
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native>& uint_plookup<Composer, Native>::operator=(uint_plookup&& other)
{
    context = other.context;
    additive_constant = other.additive_constant;
    witness_status = other.witness_status;
    accumulators = other.accumulators;
    witness_index = other.witness_index;
    return *this;
}

template <typename Context, typename Native> uint_plookup<Context, Native>::operator field_t<Context>() const
{
    normalize();
    field_t<Context> target(context);
    target.witness_index = witness_index;
    target.additive_constant = is_constant() ? fr(additive_constant) : fr::zero();
    return target;
}

template <typename Context, typename Native> uint_plookup<Context, Native>::operator byte_array<Context>() const
{
    return byte_array<Context>(static_cast<field_t<Context>>(*this), width / 8);
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::normalize() const
{
    if (!context || is_constant()) {
        return *this;
    }

    if (witness_status == WitnessStatus::WEAK_NORMALIZED) {
        accumulators = constrain_accumulators(context, witness_index);
        witness_status = WitnessStatus::OK;
    }
    return *this;
}

template <typename Composer, typename Native> uint256_t uint_plookup<Composer, Native>::get_value() const
{
    if (!context || is_constant()) {
        return additive_constant;
    }
    return (uint256_t(context->get_variable(witness_index))) & MASK;
}

template <typename Composer, typename Native> uint256_t uint_plookup<Composer, Native>::get_unbounded_value() const
{
    if (!context || is_constant()) {
        return additive_constant;
    }
    return (uint256_t(context->get_variable(witness_index)));
}

template <typename Composer, typename Native>
bool_t<Composer> uint_plookup<Composer, Native>::at(const size_t bit_index) const
{
    if (is_constant()) {
        return bool_t<Composer>(context, get_value().get_bit(bit_index));
    }
    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    const uint64_t slice_bit_position = bit_index % bits_per_limb;

    const uint32_t slice_index = accumulators[bit_index / bits_per_limb];
    const uint64_t slice_value = uint256_t(context->get_variable(slice_index)).data[0];

    const uint64_t slice_lo = slice_value % (1ULL << slice_bit_position);
    const uint64_t bit_value = (slice_value >> slice_bit_position) & 1ULL;
    const uint64_t slice_hi = slice_value >> (slice_bit_position + 1);

    const uint32_t slice_lo_idx = slice_bit_position ? context->add_variable(slice_lo) : context->zero_idx;
    const uint32_t bit_idx = context->add_variable(bit_value);
    const uint32_t slice_hi_idx =
        (slice_bit_position + 1 != bits_per_limb) ? context->add_variable(slice_hi) : context->zero_idx;

    context->create_big_add_gate({ slice_index,
                                   slice_lo_idx,
                                   bit_idx,
                                   slice_hi_idx,
                                   -1,
                                   1,
                                   (1 << slice_bit_position),
                                   (1 << (slice_bit_position + 1)),
                                   0 });

    if (slice_bit_position != 0) {
        context->create_new_range_constraint(slice_lo_idx, (1ULL << slice_bit_position) - 1);
    }
    if (slice_bit_position + 1 != bits_per_limb) {
        context->create_new_range_constraint(slice_hi_idx, (1ULL << (bits_per_limb - (slice_bit_position + 1))) - 1);
    }
    bool_t<Composer> result = witness_t<Composer>(context, bit_value);
    return result;
}

INSTANTIATE_STDLIB_ULTRA_TYPE_VA(uint_plookup, uint8_t);
INSTANTIATE_STDLIB_ULTRA_TYPE_VA(uint_plookup, uint16_t);
INSTANTIATE_STDLIB_ULTRA_TYPE_VA(uint_plookup, uint32_t);
INSTANTIATE_STDLIB_ULTRA_TYPE_VA(uint_plookup, uint64_t);

} // namespace stdlib
} // namespace plonk