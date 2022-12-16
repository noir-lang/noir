#include "uint.hpp"
#include "../composers/composers.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

/**
 * @brief  In the case of TurboPLONK, range constrain the given witness.
 */
template <typename Composer, typename Native>
std::vector<uint32_t> uint<Composer, Native>::constrain_accumulators(Composer* context,
                                                                     const uint32_t witness_index,
                                                                     const size_t num_bits,
                                                                     std::string const& msg) const
{
    if constexpr (Composer::type == waffle::PLOOKUP) {
        // TODO: manage higher bit ranges
        const auto sequence =
            plookup_read::get_lookup_accumulators(plookup::MultiTableId::UINT32_XOR,
                                                  field_t<Composer>::from_witness_index(context, witness_index),
                                                  field_t<Composer>::from_witness_index(context, context->zero_idx),
                                                  true);

        std::vector<uint32_t> out(num_accumulators());
        for (size_t i = 0; i < num_accumulators(); ++i) {
            out[i] = sequence[0][num_accumulators() - i - 1].witness_index;
        }
        return out;
    }
    return context->decompose_into_base4_accumulators(witness_index, num_bits, msg);
}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const witness_t<Composer>& witness)
    : context(witness.context)
    , additive_constant(0)
    , witness_status(WitnessStatus::OK)
    , accumulators(constrain_accumulators(
          context, witness.witness_index, width, "uint: range constraint fails in constructor of uint from witness"))
    , witness_index(accumulators[num_accumulators() - 1])
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const field_t<Composer>& value)
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
        accumulators = constrain_accumulators(
            context, norm.witness_index, width, "uint: range constraint fails in constructor of uint from field_t");
        witness_index = accumulators[num_accumulators() - 1];
    }
}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(Composer* composer, const uint256_t& value)
    : context(composer)
    , additive_constant(value)
    , witness_status(WitnessStatus::OK)
    , accumulators()
    , witness_index(IS_CONSTANT)
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const uint256_t& value)
    : context(nullptr)
    , additive_constant(value)
    , witness_status(WitnessStatus::OK)
    , accumulators()
    , witness_index(IS_CONSTANT)
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(const byte_array<Composer>& other)
    : context(other.get_context())
    , additive_constant(0)
    , witness_status(WitnessStatus::WEAK_NORMALIZED)
    , accumulators()
    , witness_index(IS_CONSTANT)
{
    ASSERT(other.bytes().size() <= sizeof(Native));
    field_t<Composer> accumulator(context, fr::zero());
    field_t<Composer> scaling_factor(context, fr::one());
    const auto bytes = other.bytes();
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
uint<Composer, Native>::uint(Composer* parent_context, const std::array<bool_t<Composer>, width>& wires)
    : uint<Composer, Native>(parent_context, std::vector<bool_t<Composer>>(wires.begin(), wires.end()))
{}

template <typename Composer, typename Native>
uint<Composer, Native>::uint(Composer* parent_context, const std::vector<bool_t<Composer>>& wires)
    : context(parent_context)
    , additive_constant(0)
    , witness_status(WitnessStatus::WEAK_NORMALIZED)
    , accumulators()
    , witness_index(IS_CONSTANT)
{
    field_t<Composer> accumulator(context, fr::zero());
    field_t<Composer> scaling_factor(context, fr::one());
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

template <typename Context, typename Native> uint<Context, Native>::operator field_t<Context>() const
{
    normalize();
    field_t<Context> target(context);
    target.witness_index = witness_index;
    target.additive_constant = is_constant() ? fr(additive_constant) : fr::zero();
    return target;
}

template <typename Context, typename Native> uint<Context, Native>::operator byte_array<Context>() const
{
    return byte_array<Context>(static_cast<field_t<Context>>(*this), width / 8);
}

/**
 * @brief Record reduction of value modulo 2^width as a constraint.
 *
 * @details This function also updates the witness and zeroes out the additive constant.
 * It does not add any range constraints.
 */
template <typename Composer, typename Native> uint<Composer, Native> uint<Composer, Native>::weak_normalize() const
{
    if (!context || is_constant()) {
        return *this;
    }
    if (witness_status == WitnessStatus::WEAK_NORMALIZED) {
        return *this;
    }

    /**
     * Constraints:
     *   witness - remainder - overflow * 2^width + (additive_constant && MASK) = 0
     * and
     *   overflow lies in {0, 1, 2}.
     *
     * Note that
     *   witness + additive_value
     * and
     *   witness + (additive_value % 2^width)
     * have the same remainder under division by 2^width.
     */

    if (witness_status == WitnessStatus::NOT_NORMALIZED) {
        const uint256_t value = get_unbounded_value();
        const uint256_t overflow = value >> width;
        const uint256_t remainder = value & MASK;
        const waffle::add_quad gate{ .a = witness_index,
                                     .b = context->zero_idx,
                                     .c = context->add_variable(remainder),
                                     .d = context->add_variable(overflow),
                                     .a_scaling = fr::one(),
                                     .b_scaling = fr::zero(),
                                     .c_scaling = fr::neg_one(),
                                     .d_scaling = -fr(CIRCUIT_UINT_MAX_PLUS_ONE),
                                     .const_scaling = (additive_constant & MASK) };

        context->create_balanced_add_gate(gate);

        witness_index = gate.c;
        witness_status = WitnessStatus::WEAK_NORMALIZED;
        additive_constant = 0;
    }
    return *this;
}

/**
 * @brief For non-constants, weakly normalize and constrain the updated witness to width-many bits.
 */
template <typename Composer, typename Native> uint<Composer, Native> uint<Composer, Native>::normalize() const
{
    if (!context || is_constant()) {
        return *this;
    }

    if (witness_status == WitnessStatus::WEAK_NORMALIZED) {
        accumulators = constrain_accumulators(
            context, witness_index, width, "uint: range constraint fails in uint normalization from weak normlized");
        witness_index = accumulators[num_accumulators() - 1];
        witness_status = WitnessStatus::OK;
    }

    if (witness_status == WitnessStatus::NOT_NORMALIZED) {
        weak_normalize();
        /**
         * constrain_accumulators will do more for PlookupComposer, but in TurboPLONK it just imposes
         * the range constraint that the witness can be expressed in width-many bits.
         *
         * The Turbo-only strategy for imposing a range constraint on a w is develop a base-4 expansion
         * of w, storing this in accumulators (just partial sums), and checking that a partial sum of a
         * fixed length actually does reproduce the witness value.
         */
        accumulators = constrain_accumulators(
            context, witness_index, width, "uint: range constraint fails in uint normalization from unnormlized");
        // This will only change the value of the uint if the range constraint fails.
        witness_index = accumulators[num_accumulators() - 1];
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

/**
 * @brief Extract the bit value at a given position.
 * @details Since we represent our uint's using quads, to extract a bit we must distinguish
 * between the case where that bit is the low bit of a quad or a high bit of a quad.
 *
 **/
template <typename Composer, typename Native> bool_t<Composer> uint<Composer, Native>::at(const size_t bit_index) const
{
    if (is_constant()) {
        return bool_t<Composer>(context, get_value().get_bit(bit_index));
    }

    if (witness_status != WitnessStatus::OK) {
        normalize();
    }

    /**
     * Calculating the position of the bit:
     * - Assume width is even and let w = width/2. There are w-many quads describing a width-bit integer.
     * We encode these in a vector of accumulators A_0, ... , A_{w-1}. Setting A_{-1} = 0.
     * - For j < w-1, the quad q_j is extracted using  as q_j = A_{w-1-j} - 4 A_{w-1-j-1}.
     * - The k-th bit lies in the k//2-th quad as the low bit (k even) or high bit (k odd).
     *
     * Therefore we need to access accumulators[pivot] and accumulators[pivot-1], where
     *      pivot = (width//2) - 1 + (bit_index//2)
     */
    const size_t pivot_index = ((width >> 1) - (bit_index >> 1UL)) - 1;
    uint32_t left_idx = (pivot_index == 0) ? context->zero_idx : accumulators[pivot_index - 1];
    uint32_t right_idx = accumulators[pivot_index];
    uint256_t quad =
        uint256_t(context->get_variable(right_idx)) - uint256_t(context->get_variable(left_idx)) * uint256_t(4);

    if constexpr (Composer::type == waffle::ComposerType::PLOOKUP) {
        uint256_t lo_bit = quad & 1;
        uint256_t hi_bit = (quad & 2) >> 1;
        // difference in quads = 0, 1, 2, 3 = delta
        // (delta - lo_bit) / 2 \in [0, 1]
        // lo_bit \in [0, 1]
        waffle::add_quad gate{
            context->add_variable(lo_bit), context->add_variable(hi_bit), right_idx, left_idx, 1, 2, -1, 4, 0,
        };
        context->create_new_range_constraint(gate.a, 1);
        context->create_new_range_constraint(gate.b, 1);
        bool_t<Composer> result;

        if ((bit_index & 1UL) == 0UL) {
            result.witness_index = gate.a;
            result.witness_bool = (lo_bit == 1) ? true : false;
        } else {
            result.witness_index = gate.b;
            result.witness_bool = (hi_bit == 1) ? true : false;
        }
        return result;
    }
    // if 'index' is odd, we want a low bit
    /**
     * Write Δ = accumulators[pivot] - 4 . accumulators[pivot - 1]. We would like to construct
     * the bit representation of Δ by imposing the constraint that Δ = lo_bit + 2 . hi_bit
     * and then return either lo_bit or hi_bit, depending on the parity of bit_index.
     * The big addition gate with bit extraction imposes the equivalent relation obtained by
     * scaling both sides by 3.
     **/

    // if 'index' is even, we want a low bit
    if ((bit_index & 1UL) == 0UL) {
        // we want a low bit
        uint256_t lo_bit = quad & 1;
        waffle::add_quad gate{ .a = context->add_variable(lo_bit),
                               .b = context->zero_idx,
                               .c = right_idx,
                               .d = left_idx,
                               .a_scaling = fr(3),
                               .b_scaling = fr::zero(),
                               .c_scaling = -fr(3),
                               .d_scaling = fr(12),
                               .const_scaling = fr::zero() };
        /** constraint:
         *    3 lo_bit + 0 * 0 - 3 a_pivot + 12 a_{pivot - 1} + 0 + 6 high bit of (A_pivot - 4 A_{pivot - 1}) == 0
         *  i.e.,
         *    lo_bit + 2 high bit of (A_pivot - 4 A_{pivot - 1}) = A_pivot - 4 A_{pivot - 1} == 0
         */
        context->create_big_add_gate_with_bit_extraction(gate);
        bool_t<Composer> result(context);
        result.witness_index = gate.a;
        result.witness_bool = (lo_bit == 1) ? true : false;
        return result;
    }

    // if 'index' is odd, we want a high bit
    uint256_t hi_bit = quad >> 1;

    waffle::add_quad gate{ .a = context->zero_idx,
                           .b = context->add_variable(hi_bit),
                           .c = right_idx,
                           .d = left_idx,
                           .a_scaling = fr::zero(),
                           .b_scaling = -fr(6),
                           .c_scaling = fr::zero(),
                           .d_scaling = fr::zero(),
                           .const_scaling = fr::zero() };

    /**
     * constraint:
     *   0 * 0 - 6 hi_bit  + 0 A_pivot + 0 * A_{pivot - 1} + 0 + 6 high bit of (A_pivot - 4 A_{pivot - 1}) == 0
     * Note: we have normalized self, so A_pivot - 4 A_{pivot - 1} is known to be in {0, 1, 2, 3}. Our protocol's
     * bit extraction gate is trusted to correctly extract 6 * (high bit c - 4d).
     */
    context->create_big_add_gate_with_bit_extraction(gate);
    bool_t<Composer> result(context);
    result.witness_index = gate.b;
    result.witness_bool = (hi_bit == 1) ? true : false;
    return result;
}

INSTANTIATE_STDLIB_BASIC_TYPE_VA(uint, uint8_t);
INSTANTIATE_STDLIB_BASIC_TYPE_VA(uint, uint16_t);
INSTANTIATE_STDLIB_BASIC_TYPE_VA(uint, uint32_t);
INSTANTIATE_STDLIB_BASIC_TYPE_VA(uint, uint64_t);

} // namespace stdlib
} // namespace plonk
