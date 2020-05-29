#include "pedersen_plookup.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

#include "../../primitives/composers/composers.hpp"
#include "../../primitives/plookup/plookup.hpp"

namespace plonk {
namespace stdlib {

using namespace barretenberg;

template <typename C> point<C> pedersen_plookup<C>::add_points(const point& p1, const point& p2, const AddType add_type)
{
    C* ctx = p1.x.context ? p1.x.context : (p1.y.context ? p1.y.context : (p2.x.context ? p2.x.context : p2.y.context));
    grumpkin::fq x_1_raw = p1.x.get_value();
    grumpkin::fq y_1_raw = p1.y.get_value();
    grumpkin::fq x_2_raw = p2.x.get_value();
    grumpkin::fq y_2_raw = p2.y.get_value();
    grumpkin::fq endomorphism_coefficient = 1;
    grumpkin::fq sign_coefficient = 1;
    switch (add_type) {
    case ONE: {
        break;
    }
    case LAMBDA: {
        endomorphism_coefficient = grumpkin::fq::beta();
        x_2_raw *= endomorphism_coefficient;
        break;
    }
    case ONE_PLUS_LAMBDA: {
        endomorphism_coefficient = grumpkin::fq::beta().sqr();
        sign_coefficient = -1;
        x_2_raw *= endomorphism_coefficient;
        y_2_raw = -y_2_raw;
        break;
    }
    }

    grumpkin::fq lambda_raw = (y_2_raw - y_1_raw) / (x_2_raw - x_1_raw);
    grumpkin::fq x_3_raw = lambda_raw.sqr() - x_2_raw - x_1_raw;
    grumpkin::fq y_3_raw = lambda_raw * (x_1_raw - x_3_raw) - y_1_raw;

    bool p1_constant = (p1.x.witness_index == UINT32_MAX) && (p1.y.witness_index == UINT32_MAX);
    bool p2_constant = (p2.x.witness_index == UINT32_MAX) && (p2.y.witness_index == UINT32_MAX);

    if (p1_constant && p2_constant) {
        return point{ field_t(ctx, x_3_raw), field_t(ctx, y_3_raw) };
    }
    if (p1_constant || p2_constant) {
        field_t lambda = (p2.y - p1.y) / (p2.x - p1.x);
        field_t x_3 = lambda.madd(lambda, -(p2.x + p1.x));
        field_t y_3 = lambda.madd(p1.x - x_3, p1.y);
        return point{ x_3, y_3 };
    }

    point p3{ witness_t(ctx, x_3_raw), witness_t(ctx, y_3_raw) };

    waffle::ecc_add_gate add_gate =
        waffle::ecc_add_gate{ p1.x.witness_index, p1.y.witness_index, p2.x.witness_index,       p2.y.witness_index,
                              p3.x.witness_index, p3.y.witness_index, endomorphism_coefficient, sign_coefficient };
    ctx->create_ecc_add_gate(add_gate);

    return p3;
}

template <typename C> point<C> pedersen_plookup<C>::hash_single(const field_t& in, const bool parity)
{
    field_t scalar = in.normalize();

    std::array<std::vector<field_t>, 3> sequence;
    if (parity) {
        sequence = plookup::read_sequence_from_table(waffle::PLookupMultiTableId::PEDERSEN_RIGHT, scalar);
    } else {
        sequence = plookup::read_sequence_from_table(waffle::PLookupMultiTableId::PEDERSEN_LEFT, scalar);
    }

    const size_t num_lookups = sequence[0].size();

    point p1{ sequence[1][num_lookups - 1], sequence[2][num_lookups - 1] };

    for (size_t i = 0; i < num_lookups - 1; ++i) {
        point p2 = { sequence[1][i], sequence[2][i] };
        AddType type = (i % 3 == 0) ? LAMBDA : (i % 3 == 1 ? ONE : ONE_PLUS_LAMBDA);
        point p3 = add_points(p1, p2, type);
        p1 = p3;
    }

    return p1;
}

template <typename C> point<C> pedersen_plookup<C>::compress_to_point(const field_t& left, const field_t& right)
{
    auto p2 = hash_single(left, false);
    auto p1 = hash_single(right, true);

    return add_points(p1, p2);
}

template <typename C> field_t<C> pedersen_plookup<C>::compress(const field_t& left, const field_t& right)
{
    return compress_to_point(left, right).x;
}

template <typename C> point<C> pedersen_plookup<C>::encrypt(const std::vector<field_t>& inputs)
{
    const size_t num_inputs = inputs.size();

    size_t num_tree_levels = numeric::get_msb(num_inputs) + 1;
    if (1UL << num_tree_levels < num_inputs) {
        ++num_tree_levels;
    }

    std::vector<field_t> previous_leaves(inputs.begin(), inputs.end());

    for (size_t i = 0; i < num_tree_levels - 1; ++i) {
        const size_t num_leaves = 1UL << (num_tree_levels - i);
        std::vector<field_t> current_leaves;
        for (size_t j = 0; j < num_leaves; j += 2) {
            field_t left;
            field_t right;
            if (j < previous_leaves.size()) {
                left = previous_leaves[j];
            } else {
                left = 0;
            }

            if ((j + 1) < previous_leaves.size()) {
                right = previous_leaves[j + 1];
            } else {
                right = 0;
            }

            current_leaves.push_back(compress(left, right));
        }

        previous_leaves.resize(current_leaves.size());
        std::copy(current_leaves.begin(), current_leaves.end(), previous_leaves.begin());
    }

    return compress_to_point(previous_leaves[0], previous_leaves[1]);
}

template <typename C> field_t<C> pedersen_plookup<C>::compress(const std::vector<field_t>& inputs)
{
    return encrypt(inputs).x;
}

template class pedersen_plookup<waffle::PLookupComposer>;

} // namespace stdlib
} // namespace plonk