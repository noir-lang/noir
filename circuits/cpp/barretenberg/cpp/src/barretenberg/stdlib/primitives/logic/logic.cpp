#include "logic.hpp"

#include "../composers/composers.hpp"
#include "../plookup/plookup.hpp"

namespace proof_system::plonk::stdlib {

/**
 * @brief A logical AND or XOR over a variable number of bits.
 *
 * @details Defaults to basic Composer method if not using plookup-compatible composer
 *
 * @tparam Composer
 * @param a
 * @param b
 * @param num_bits
 * @param is_xor_gate
 * @return field_t<Composer>
 */
template <typename Composer>
field_t<Composer> logic<Composer>::create_logic_constraint(field_pt& a, field_pt& b, size_t num_bits, bool is_xor_gate)
{
    // can't extend past field size!
    ASSERT(num_bits < 254);
    if (a.is_constant() && b.is_constant()) {
        uint256_t a_native(a.get_value());
        uint256_t b_native(b.get_value());
        uint256_t c_native = is_xor_gate ? (a_native ^ b_native) : (a_native & b_native);
        return field_t<Composer>(c_native);
    }
    if (a.is_constant() && !b.is_constant()) {
        Composer* ctx = b.get_context();
        uint256_t a_native(a.get_value());
        field_t<Composer> a_witness = field_pt::from_witness_index(ctx, ctx->put_constant_variable(a_native));
        return create_logic_constraint(a_witness, b, num_bits, is_xor_gate);
    }
    if (!a.is_constant() && b.is_constant()) {
        Composer* ctx = a.get_context();
        uint256_t b_native(b.get_value());
        field_pt b_witness = field_pt::from_witness_index(ctx, ctx->put_constant_variable(b_native));
        return create_logic_constraint(a, b_witness, num_bits, is_xor_gate);
    }
    if constexpr (Composer::type == ComposerType::PLOOKUP) {
        Composer* ctx = a.get_context();

        const size_t num_chunks = (num_bits / 32) + ((num_bits % 32 == 0) ? 0 : 1);
        uint256_t left(a.get_value());
        uint256_t right(b.get_value());

        field_pt res(ctx, 0);
        for (size_t i = 0; i < num_chunks; ++i) {
            uint256_t left_chunk = left & ((uint256_t(1) << 32) - 1);
            uint256_t right_chunk = right & ((uint256_t(1) << 32) - 1);

            const field_pt a_chunk = witness_pt(ctx, left_chunk);
            const field_pt b_chunk = witness_pt(ctx, right_chunk);

            field_pt result_chunk = 0;
            if (is_xor_gate) {
                result_chunk =
                    stdlib::plookup_read::read_from_2_to_1_table(plookup::MultiTableId::UINT32_XOR, a_chunk, b_chunk);
            } else {
                result_chunk =
                    stdlib::plookup_read::read_from_2_to_1_table(plookup::MultiTableId::UINT32_AND, a_chunk, b_chunk);
            }

            uint256_t scaling_factor = uint256_t(1) << (32 * i);
            res += result_chunk * scaling_factor;

            if (i == num_chunks - 1) {
                const size_t final_num_bits = num_bits - (i * 32);
                if (final_num_bits != 32) {
                    ctx->create_range_constraint(a_chunk.witness_index, final_num_bits, "bad range on a");
                    ctx->create_range_constraint(b_chunk.witness_index, final_num_bits, "bad range on b");
                }
            }

            left = left >> 32;
            right = right >> 32;
        }

        return res;
    } else {
        Composer* ctx = a.get_context();
        auto accumulator_triple = ctx->create_logic_constraint(
            a.normalize().get_witness_index(), b.normalize().get_witness_index(), num_bits, is_xor_gate);
        auto out_idx = accumulator_triple.out[accumulator_triple.out.size() - 1];
        return field_t<Composer>::from_witness_index(ctx, out_idx);
    }
}
INSTANTIATE_STDLIB_TYPE(logic);
} // namespace proof_system::plonk::stdlib
