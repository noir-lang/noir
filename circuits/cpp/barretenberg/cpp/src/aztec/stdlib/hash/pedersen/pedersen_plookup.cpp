#include "pedersen_plookup.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

#include <plonk/composer/plookup_tables/types.hpp>
#include "../../primitives/composers/composers.hpp"
#include "../../primitives/plookup/plookup.hpp"

using namespace bonk;

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
    grumpkin::fq beta = grumpkin::fq::cube_root_of_unity();
    switch (add_type) {
    case ONE: {
        break;
    }
    case LAMBDA: {
        endomorphism_coefficient = beta;
        x_2_raw *= endomorphism_coefficient;
        break;
    }
    case ONE_PLUS_LAMBDA: {
        endomorphism_coefficient = beta.sqr();
        sign_coefficient = -1;
        x_2_raw *= endomorphism_coefficient;
        y_2_raw = -y_2_raw;
        break;
    }
    }

    grumpkin::fq lambda_raw = (y_2_raw - y_1_raw) / (x_2_raw - x_1_raw);
    grumpkin::fq x_3_raw = lambda_raw.sqr() - x_2_raw - x_1_raw;
    grumpkin::fq y_3_raw = lambda_raw * (x_1_raw - x_3_raw) - y_1_raw;

    bool p1_constant = (p1.x.witness_index == IS_CONSTANT) && (p1.y.witness_index == IS_CONSTANT);
    bool p2_constant = (p2.x.witness_index == IS_CONSTANT) && (p2.y.witness_index == IS_CONSTANT);

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

    ecc_add_gate add_gate =
        ecc_add_gate{ p1.x.witness_index, p1.y.witness_index, p2.x.witness_index,       p2.y.witness_index,
                      p3.x.witness_index, p3.y.witness_index, endomorphism_coefficient, sign_coefficient };
    ctx->create_ecc_add_gate(add_gate);

    return p3;
}

template <typename C> point<C> pedersen_plookup<C>::hash_single(const field_t& scalar, const bool parity)
{
    if (scalar.is_constant()) {
        C* ctx = scalar.get_context();
        const auto hash_native = crypto::pedersen::lookup::hash_single(scalar.get_value(), parity).normalize();
        return { field_t(ctx, hash_native.x), field_t(ctx, hash_native.y) };
    }

    // Slice the input scalar in lower 126 and higher 128 bits.
    C* ctx = scalar.get_context();
    const field_t y_hi = witness_t(ctx, uint256_t(scalar.get_value()).slice(126, 256));
    const field_t y_lo = witness_t(ctx, uint256_t(scalar.get_value()).slice(0, 126));

    ReadData<field_t> lookup_hi, lookup_lo;
    if (parity) {
        lookup_lo = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_RIGHT_LO, y_lo);
        lookup_hi = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_RIGHT_HI, y_hi);
    } else {
        lookup_lo = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_LEFT_LO, y_lo);
        lookup_hi = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_LEFT_HI, y_hi);
    }

    // Check if (r_hi - y_hi) is 128 bits and if (r_hi - y_hi) == 0, then
    // (r_lo - y_lo) must be 126 bits.
    constexpr uint256_t modulus = fr::modulus;
    const field_t r_lo = witness_t(ctx, modulus.slice(0, 126));
    const field_t r_hi = witness_t(ctx, modulus.slice(126, 256));

    const field_t term_hi = r_hi - y_hi;
    const field_t term_lo = (r_lo - y_lo) * field_t(term_hi == field_t(0));
    term_hi.normalize().create_range_constraint(128);
    term_lo.normalize().create_range_constraint(126);

    const size_t num_lookups_lo = lookup_lo[ColumnIdx::C1].size();
    const size_t num_lookups_hi = lookup_hi[ColumnIdx::C1].size();

    point p1{ lookup_lo[ColumnIdx::C2][1], lookup_lo[ColumnIdx::C3][1] };
    point p2{ lookup_lo[ColumnIdx::C2][0], lookup_lo[ColumnIdx::C3][0] };
    point res = add_points(p1, p2, LAMBDA);

    for (size_t i = 2; i < num_lookups_lo; ++i) {
        point p2 = { lookup_lo[ColumnIdx::C2][i], lookup_lo[ColumnIdx::C3][i] };
        AddType basic_type = (i % 2 == 0) ? LAMBDA : ONE;
        point p3 = add_points(res, p2, basic_type);
        res = p3;
    }

    for (size_t i = 0; i < num_lookups_hi; ++i) {
        point p2 = { lookup_hi[ColumnIdx::C2][i], lookup_hi[ColumnIdx::C3][i] };
        AddType basic_type = (i % 2 == 0) ? LAMBDA : ONE;
        point p3 = add_points(res, p2, basic_type);
        res = p3;
    }

    return res;
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

template <typename C>
point<C> pedersen_plookup<C>::merkle_damgard_compress(const std::vector<field_t>& inputs, const field_t& iv)
{
    if (inputs.size() == 0) {
        return point{ 0, 0 };
    }

    auto result = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_IV, iv)[ColumnIdx::C2][0];
    auto num_inputs = inputs.size();
    for (size_t i = 0; i < num_inputs; i++) {
        result = compress(result, inputs[i]);
    }

    return compress_to_point(result, field_t(num_inputs));
}

template <typename C> point<C> pedersen_plookup<C>::commit(const std::vector<field_t>& inputs, const size_t hash_index)
{
    return merkle_damgard_compress(inputs, field_t(hash_index));
}

template <typename C>
field_t<C> pedersen_plookup<C>::compress(const std::vector<field_t>& inputs, const size_t hash_index)
{
    return commit(inputs, hash_index).x;
}

template class pedersen_plookup<waffle::UltraComposer>;

} // namespace stdlib
} // namespace plonk