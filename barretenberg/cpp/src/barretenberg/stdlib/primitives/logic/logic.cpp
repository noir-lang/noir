#include "logic.hpp"
#include "../circuit_builders/circuit_builders.hpp"
#include "../plookup/plookup.hpp"
#include "barretenberg/common/assert.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include <cstddef>

namespace bb::stdlib {

/**
 * @brief A logical AND or XOR over a variable number of bits.
 *
 * @details Defaults to basic Builder method if not using plookup-compatible builder. If the left and right operands
 * are larger than num_bit, the result will be truncated to num_bits. However, the two operands could be
 * range-constrained to num_bits before the call, which would remove the need to range constrain inside this function.
 *
 * @tparam Builder
 * @param a
 * @param b
 * @param num_bits
 * @param is_xor_gate
 * @return field_t<Builder>
 */
template <typename Builder>
field_t<Builder> logic<Builder>::create_logic_constraint(
    field_pt& a,
    field_pt& b,
    size_t num_bits,
    bool is_xor_gate,
    const std::function<std::pair<uint256_t, uint256_t>(uint256_t, uint256_t, size_t)>& get_chunk)
{
    // ensure the number of bits doesn't exceed field size and is not negatove
    ASSERT(num_bits < 254);
    ASSERT(num_bits > 0);

    if (a.is_constant() && b.is_constant()) {
        uint256_t a_native(a.get_value());
        uint256_t b_native(b.get_value());
        uint256_t c_native = is_xor_gate ? (a_native ^ b_native) : (a_native & b_native);
        return field_t<Builder>(c_native);
    }
    if (a.is_constant() && !b.is_constant()) {
        Builder* ctx = b.get_context();
        uint256_t a_native(a.get_value());
        field_pt a_witness = field_pt::from_witness_index(ctx, ctx->put_constant_variable(a_native));
        return create_logic_constraint(a_witness, b, num_bits, is_xor_gate, get_chunk);
    }
    if (!a.is_constant() && b.is_constant()) {
        Builder* ctx = a.get_context();
        uint256_t b_native(b.get_value());
        field_pt b_witness = field_pt::from_witness_index(ctx, ctx->put_constant_variable(b_native));
        return create_logic_constraint(a, b_witness, num_bits, is_xor_gate, get_chunk);
    }
    if constexpr (HasPlookup<Builder>) {
        Builder* ctx = a.get_context();

        const size_t num_chunks = (num_bits / 32) + ((num_bits % 32 == 0) ? 0 : 1);
        auto left((uint256_t)a.get_value());
        auto right((uint256_t)b.get_value());

        field_pt a_accumulator(bb::fr::zero());
        field_pt b_accumulator(bb::fr::zero());

        field_pt res(ctx, 0);
        for (size_t i = 0; i < num_chunks; ++i) {
            size_t chunk_size = (i != num_chunks - 1) ? 32 : num_bits - i * 32;
            auto [left_chunk, right_chunk] = get_chunk(left, right, chunk_size);

            field_pt a_chunk = witness_pt(ctx, left_chunk);
            field_pt b_chunk = witness_pt(ctx, right_chunk);
            field_pt result_chunk = 0;
            if (is_xor_gate) {
                result_chunk = stdlib::plookup_read<Builder>::read_from_2_to_1_table(
                    plookup::MultiTableId::UINT32_XOR, a_chunk, b_chunk);

            } else {
                result_chunk = stdlib::plookup_read<Builder>::read_from_2_to_1_table(
                    plookup::MultiTableId::UINT32_AND, a_chunk, b_chunk);
            }

            auto scaling_factor = uint256_t(1) << (32 * i);
            a_accumulator += a_chunk * scaling_factor;
            b_accumulator += b_chunk * scaling_factor;

            if (chunk_size != 32) {
                ctx->create_range_constraint(
                    a_chunk.witness_index, chunk_size, "stdlib logic: bad range on final chunk of left operand");
                ctx->create_range_constraint(
                    b_chunk.witness_index, chunk_size, "stdlib logic: bad range on final chunk of right operand");
            }

            res += result_chunk * scaling_factor;

            left = left >> 32;
            right = right >> 32;
        }
        field_pt a_slice = a.slice(static_cast<uint8_t>(num_bits - 1), 0)[1];
        field_pt b_slice = b.slice(static_cast<uint8_t>(num_bits - 1), 0)[1];
        a_slice.assert_equal(a_accumulator, "stdlib logic: failed to reconstruct left operand");
        b_slice.assert_equal(b_accumulator, "stdlib logic: failed to reconstruct right operand");

        return res;
    } else {
        // If the builder doesn't have lookups we call the expensive logic constraint gate
        // which creates constraints for each bit. We only create constraints up to num_bits.
        Builder* ctx = a.get_context();
        field_pt a_slice = a.slice(static_cast<uint8_t>(num_bits - 1), 0)[1];
        field_pt b_slice = b.slice(static_cast<uint8_t>(num_bits - 1), 0)[1];
        auto accumulator_triple = ctx->create_logic_constraint(
            a_slice.normalize().get_witness_index(), b_slice.normalize().get_witness_index(), num_bits, is_xor_gate);
        auto out_idx = accumulator_triple.out[accumulator_triple.out.size() - 1];
        return field_t<Builder>::from_witness_index(ctx, out_idx);
    }
}
template class logic<bb::StandardCircuitBuilder>;
template class logic<bb::UltraCircuitBuilder>;
template class logic<bb::GoblinUltraCircuitBuilder>;
} // namespace bb::stdlib
