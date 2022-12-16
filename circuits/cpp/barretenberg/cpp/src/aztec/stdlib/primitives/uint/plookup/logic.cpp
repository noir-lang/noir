#include "../../composers/composers.hpp"
#include "uint.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::operator&(const uint_plookup& other) const
{
    return logic_operator(other, LogicOp::AND);
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::operator^(const uint_plookup& other) const
{
    return logic_operator(other, LogicOp::XOR);
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::operator|(const uint_plookup& other) const
{
    return (*this + other) - (*this & other);
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::operator~() const
{
    return uint_plookup(context, MASK) - *this;
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::operator>>(const size_t shift) const
{
    if (shift >= width) {
        return uint_plookup(context, 0);
    }
    if (is_constant()) {
        return uint_plookup(context, (additive_constant >> shift) & MASK);
    }

    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    if (shift == 0) {
        return *this;
    }

    uint64_t bits_per_hi_limb;
    // last limb will not likely bit `bits_per_limb`. Need to be careful with our range check
    if (shift >= ((width / bits_per_limb) * bits_per_limb)) {
        bits_per_hi_limb = width % bits_per_limb;
    } else {
        bits_per_hi_limb = bits_per_limb;
    }
    const uint64_t slice_bit_position = shift % bits_per_limb;
    const size_t accumulator_index = shift / bits_per_limb;
    const uint32_t slice_index = accumulators[accumulator_index];
    const uint64_t slice_value = uint256_t(context->get_variable(slice_index)).data[0];

    const uint64_t slice_lo = slice_value % (1ULL << slice_bit_position);
    const uint64_t slice_hi = slice_value >> slice_bit_position;
    const uint32_t slice_lo_idx = slice_bit_position ? context->add_variable(slice_lo) : context->zero_idx;
    const uint32_t slice_hi_idx =
        (slice_bit_position != bits_per_limb) ? context->add_variable(slice_hi) : context->zero_idx;

    context->create_big_add_gate(
        { slice_index, slice_lo_idx, context->zero_idx, slice_hi_idx, -1, 1, 0, (1 << slice_bit_position), 0 });

    if (slice_bit_position != 0) {
        context->create_new_range_constraint(slice_lo_idx, (1ULL << slice_bit_position) - 1);
    }
    context->create_new_range_constraint(slice_hi_idx, (1ULL << (bits_per_hi_limb - slice_bit_position)) - 1);
    std::vector<field_t<Composer>> sublimbs;
    sublimbs.emplace_back(field_t<Composer>::from_witness_index(context, slice_hi_idx));

    const size_t start = accumulator_index + 1;
    field_t<Composer> coefficient(context, uint64_t(1ULL << (start * bits_per_limb - shift)));
    field_t<Composer> shifter(context, uint64_t(1ULL << bits_per_limb));
    for (size_t i = accumulator_index + 1; i < num_accumulators(); ++i) {
        sublimbs.emplace_back(field_t<Composer>::from_witness_index(context, accumulators[i]) *
                              field_t<Composer>(coefficient));
        coefficient *= shifter;
    }

    uint32_t result_index = field_t<Composer>::accumulate(sublimbs).normalize().get_witness_index();
    uint_plookup result(context);
    result.witness_index = result_index;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;
    return result;
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::operator<<(const size_t shift) const
{
    if (shift >= width) {
        return uint_plookup(context, 0);
    }
    if (is_constant()) {
        return uint_plookup(context, (additive_constant << shift) & MASK);
    }

    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    if (shift == 0) {
        return *this;
    }

    uint64_t slice_bit_position;
    size_t accumulator_index;
    size_t bits_per_hi_limb;
    // most significant limb is only 2 bits long (for u32), need to be careful about which slice we index,
    // and how large the range check is on our hi limb
    if (shift < (width - ((width / bits_per_limb) * bits_per_limb))) {
        bits_per_hi_limb = width % bits_per_limb;
        slice_bit_position = bits_per_hi_limb - (shift % bits_per_hi_limb);
        accumulator_index = num_accumulators() - 1;
    } else {
        const size_t offset = width % bits_per_limb;
        slice_bit_position = bits_per_limb - ((shift - offset) % bits_per_limb);
        accumulator_index = num_accumulators() - 2 - ((shift - offset) / bits_per_limb);
        bits_per_hi_limb = bits_per_limb;
    }

    const uint32_t slice_index = accumulators[accumulator_index];
    const uint64_t slice_value = uint256_t(context->get_variable(slice_index)).data[0];

    const uint64_t slice_lo = slice_value % (1ULL << slice_bit_position);
    const uint64_t slice_hi = slice_value >> slice_bit_position;
    const uint32_t slice_lo_idx = slice_bit_position ? context->add_variable(slice_lo) : context->zero_idx;
    const uint32_t slice_hi_idx =
        (slice_bit_position != bits_per_hi_limb) ? context->add_variable(slice_hi) : context->zero_idx;

    context->create_big_add_gate(
        { slice_index, slice_lo_idx, context->zero_idx, slice_hi_idx, -1, 1, 0, (1 << slice_bit_position), 0 });

    context->create_new_range_constraint(slice_lo_idx, (1ULL << slice_bit_position) - 1);

    if (slice_bit_position != bits_per_limb) {
        context->create_new_range_constraint(slice_hi_idx, (1ULL << (bits_per_hi_limb - slice_bit_position)) - 1);
    }

    std::vector<field_t<Composer>> sublimbs;
    sublimbs.emplace_back(field_t<Composer>::from_witness_index(context, slice_lo_idx) *
                          field_t<Composer>(context, 1ULL << ((accumulator_index)*bits_per_limb + shift)));

    field_t<Composer> coefficient(context, uint64_t(1ULL << shift));
    field_t<Composer> shifter(context, uint64_t(1ULL << bits_per_limb));
    for (size_t i = 0; i < accumulator_index; ++i) {
        sublimbs.emplace_back(field_t<Composer>::from_witness_index(context, accumulators[i]) *
                              field_t<Composer>(coefficient));
        coefficient *= shifter;
    }

    uint32_t result_index = field_t<Composer>::accumulate(sublimbs).normalize().get_witness_index();
    uint_plookup result(context);
    result.witness_index = result_index;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;
    return result;
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::ror(const size_t target_rotation) const
{
    const size_t rotation = target_rotation & (width - 1);

    const auto rotate = [](const uint256_t input, const uint64_t rot) {
        uint256_t r0 = (input >> rot);
        uint256_t r1 = (input << (width - rot)) & MASK;
        return (rot > 0) ? (r0 + r1) : input;
    };

    if (is_constant()) {
        return uint_plookup(context, rotate(additive_constant, rotation));
    }

    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    if (rotation == 0) {
        return *this;
    }

    const size_t shift = rotation;
    uint64_t bits_per_hi_limb;
    // last limb will not likely bit `bits_per_limb`. Need to be careful with our range check
    if (shift >= ((width / bits_per_limb) * bits_per_limb)) {
        bits_per_hi_limb = width % bits_per_limb;
    } else {
        bits_per_hi_limb = bits_per_limb;
    }
    const uint64_t slice_bit_position = shift % bits_per_limb;
    const size_t accumulator_index = shift / bits_per_limb;
    const uint32_t slice_index = accumulators[accumulator_index];
    const uint64_t slice_value = uint256_t(context->get_variable(slice_index)).data[0];

    const uint64_t slice_lo = slice_value % (1ULL << slice_bit_position);
    const uint64_t slice_hi = slice_value >> slice_bit_position;
    const uint32_t slice_lo_idx = slice_bit_position ? context->add_variable(slice_lo) : context->zero_idx;
    const uint32_t slice_hi_idx =
        (slice_bit_position != bits_per_limb) ? context->add_variable(slice_hi) : context->zero_idx;

    context->create_big_add_gate(
        { slice_index, slice_lo_idx, context->zero_idx, slice_hi_idx, -1, 1, 0, (1 << slice_bit_position), 0 });

    if (slice_bit_position != 0) {
        context->create_new_range_constraint(slice_lo_idx, (1ULL << slice_bit_position) - 1);
    }
    context->create_new_range_constraint(slice_hi_idx, (1ULL << (bits_per_hi_limb - slice_bit_position)) - 1);
    std::vector<field_t<Composer>> sublimbs;
    sublimbs.emplace_back(field_t<Composer>::from_witness_index(context, slice_hi_idx));

    const size_t start = accumulator_index + 1;
    field_t<Composer> coefficient(context, uint64_t(1ULL << (start * bits_per_limb - shift)));
    field_t<Composer> shifter(context, uint64_t(1ULL << bits_per_limb));
    for (size_t i = accumulator_index + 1; i < num_accumulators(); ++i) {
        sublimbs.emplace_back(field_t<Composer>::from_witness_index(context, accumulators[i]) *
                              field_t<Composer>(coefficient));
        coefficient *= shifter;
    }

    coefficient = field_t<Composer>(context, uint64_t(1ULL << (width - shift)));
    for (size_t i = 0; i < accumulator_index; ++i) {
        sublimbs.emplace_back(field_t<Composer>::from_witness_index(context, accumulators[i]) *
                              field_t<Composer>(coefficient));
        coefficient *= shifter;
    }
    sublimbs.emplace_back(field_t<Composer>::from_witness_index(context, slice_lo_idx) *
                          field_t<Composer>(coefficient));

    uint32_t result_index = field_t<Composer>::accumulate(sublimbs).normalize().get_witness_index();
    uint_plookup result(context);
    result.witness_index = result_index;
    result.witness_status = WitnessStatus::WEAK_NORMALIZED;
    return result;
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::rol(const size_t target_rotation) const
{
    return ror(width - (target_rotation & (width - 1)));
}

template <typename Composer, typename Native>
uint_plookup<Composer, Native> uint_plookup<Composer, Native>::logic_operator(const uint_plookup& other,
                                                                              const LogicOp op_type) const
{
    Composer* ctx = (context == nullptr) ? other.context : context;

    // we need to ensure that we can decompose our integers into (width / 2) quads
    // we don't need to completely normalize, however, as our quaternary decomposition will do that by default
    const uint256_t lhs = get_value();
    const uint256_t rhs = other.get_value();
    uint256_t out = 0;

    switch (op_type) {
    case AND: {
        out = lhs & rhs;
        break;
    }
    case XOR: {
        out = lhs ^ rhs;
        break;
    }
    default: {
    }
    }

    if (is_constant() && other.is_constant()) {
        return uint_plookup<Composer, Native>(ctx, out);
    }

    ReadData<field_t<Composer>> lookup;
    if (op_type == XOR) {
        lookup = plookup_read::get_lookup_accumulators(
            MultiTableId::UINT32_XOR, field_t<Composer>(*this), field_t<Composer>(other), true);
    } else {
        lookup = plookup_read::get_lookup_accumulators(
            MultiTableId::UINT32_AND, field_t<Composer>(*this), field_t<Composer>(other), true);
    }
    uint_plookup<Composer, Native> result(ctx);
    // result.accumulators.resize(num_accumulators());
    field_t<Composer> scaling_factor(context, barretenberg::fr(1ULL << bits_per_limb));

    // N.B. THIS LOOP ONLY WORKS IF THE LOGIC TABLE SLICE SIZE IS HALF THAT OF `bits_per_limb`
    for (size_t i = 0; i < num_accumulators(); ++i) {

        /**
         * we can extract a slice value, by taking the relative difference between accumulating sums.
         * each table row sums a 6-bit slice into an accumulator, we need to take the difference between slices in jumps
         *of 2, to get a 12-bit slice
         *
         * If our output limbs are b0, b1, b2, b3, b4, b5, our lookup[ColumnIdx::C3] values represent:
         * (where X = 2^6)
         *   | c0 | b0 + X.b1 + X.X.b2 + X.X.X.b3 + X.X.X.X.b4 + X.X.X.X.X.b5
         *   | c1 |        b1 +   X.b2 +   X.X.b3 +   X.X.X.b4 +   X.X.X.X.b5
         *   | c2 |                 b2 +   X.  b3 +     X.X.b4 +     X.X.X.b5
         *   | c3 |                            b3 +       X.b4 +       X.X.b5
         *   | c4 |                                         b4 +         X.b5
         *   | c5 |                                                        b5
         *
         *
         * We want in our accumulators:
         *
         *   | acc[0] | c0 - X.X.c2 |
         *   | acc[1] | c2 - X.X.c4 |
         *   | acc[2] | c4          |
         **/

        if (i != (num_accumulators() - 1)) {
            result.accumulators.emplace_back(
                (lookup[ColumnIdx::C3][2 * i] - (lookup[ColumnIdx::C3][2 * (i + 1)] * scaling_factor)).witness_index);
        } else {
            result.accumulators.emplace_back(lookup[ColumnIdx::C3][2 * (num_accumulators() - 1)].witness_index);
        }
    }

    result.witness_index = lookup[ColumnIdx::C3][0].get_witness_index();
    result.witness_status = WitnessStatus::OK;
    return result;
}

template class uint_plookup<waffle::UltraComposer, uint8_t>;
template class uint_plookup<waffle::UltraComposer, uint16_t>;
template class uint_plookup<waffle::UltraComposer, uint32_t>;
template class uint_plookup<waffle::UltraComposer, uint64_t>;

} // namespace stdlib
} // namespace plonk