#include "pedersen_plookup.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

#include "../../primitives/composers/composers.hpp"
#include "../../primitives/plookup/plookup.hpp"

namespace plonk {
namespace stdlib {

using namespace barretenberg;

template <typename C> point<C> pedersen_plookup<C>::hash_single(const field_t& in, const bool parity)
{
    C* ctx = in.context;
    ASSERT(ctx != nullptr);

    field_t scalar = in.normalize();

    std::array<std::vector<field_t>, 3> sequence;
    if (parity) {
        sequence = plookup::read_sequence_from_table(waffle::PLookupMultiTableId::PEDERSEN_RIGHT, scalar);
    } else {
        sequence = plookup::read_sequence_from_table(waffle::PLookupMultiTableId::PEDERSEN_LEFT, scalar);
    }

    const size_t num_lookups = sequence[0].size();

    field_t x_1 = sequence[1][num_lookups - 1];
    field_t y_1 = sequence[2][num_lookups - 1];
    field_t x_3;
    field_t y_3;
    for (size_t i = 0; i < num_lookups - 1; ++i) {
        field_t x_2 = sequence[1][i];
        field_t y_2 = sequence[2][i];

        grumpkin::fq x_1_raw = x_1.get_value();
        grumpkin::fq y_1_raw = y_1.get_value();
        grumpkin::fq x_2_raw = x_2.get_value();
        grumpkin::fq y_2_raw = y_2.get_value();
        grumpkin::fq endomorphism_coefficient = 1;
        grumpkin::fq sign_coefficient = 1;
        if (i % 3 == 0) {
            endomorphism_coefficient = grumpkin::fq::beta();
            x_2_raw *= endomorphism_coefficient;
        } else if (i % 3 == 2) {
            endomorphism_coefficient = grumpkin::fq::beta().sqr();
            sign_coefficient = -1;
            x_2_raw *= endomorphism_coefficient;
            y_2_raw = -y_2_raw;
        }
        grumpkin::fq lambda_raw = (y_2_raw - y_1_raw) / (x_2_raw - x_1_raw);
        grumpkin::fq x_3_raw = lambda_raw.sqr() - x_2_raw - x_1_raw;
        grumpkin::fq y_3_raw = lambda_raw * (x_1_raw - x_3_raw) - y_1_raw;

        x_3 = witness_t(ctx, x_3_raw);
        y_3 = witness_t(ctx, y_3_raw);

        waffle::ecc_add_gate add_gate =
            waffle::ecc_add_gate{ x_1.witness_index, y_1.witness_index, x_2.witness_index,        y_2.witness_index,
                                  x_3.witness_index, y_3.witness_index, endomorphism_coefficient, sign_coefficient };
        ctx->create_ecc_add_gate(add_gate, true);

        x_1 = x_3;
        y_1 = y_3;
    }

    return { x_3, y_3 };
}

template <typename C> field_t<C> pedersen_plookup<C>::compress(const field_t& left, const field_t& right)
{
    // TODO HANDLE CONSTANT OPERANDS
    C* ctx = left.context;
    ASSERT(ctx != nullptr);

    auto [x_2, y_2] = hash_single(left, false);
    auto [x_1, y_1] = hash_single(right, true);

    grumpkin::fq x_1_raw = x_1.get_value();
    grumpkin::fq y_1_raw = y_1.get_value();
    grumpkin::fq x_2_raw = x_2.get_value();
    grumpkin::fq y_2_raw = y_2.get_value();
    grumpkin::fq lambda_raw = (y_2_raw - y_1_raw) / (x_2_raw - x_1_raw);
    grumpkin::fq x_3_raw = lambda_raw.sqr() - x_2_raw - x_1_raw;
    grumpkin::fq y_3_raw = lambda_raw * (x_1_raw - x_3_raw) - y_1_raw;

    field_t x_3 = witness_t(ctx, x_3_raw);
    field_t y_3 = witness_t(ctx, y_3_raw);

    waffle::ecc_add_gate add_gate = waffle::ecc_add_gate{
        x_1.witness_index,
        y_1.witness_index,
        x_2.witness_index,
        y_2.witness_index,
        x_3.witness_index,
        y_3.witness_index,
        1,
        1,
    };
    ctx->create_ecc_add_gate(add_gate, true);

    return x_3;
}
template class pedersen_plookup<waffle::PLookupComposer>;

} // namespace stdlib
} // namespace plonk