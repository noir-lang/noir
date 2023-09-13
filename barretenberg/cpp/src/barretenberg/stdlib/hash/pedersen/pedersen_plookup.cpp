#include "pedersen_plookup.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

#include "../../primitives/plookup/plookup.hpp"
#include "barretenberg/proof_system/plookup_tables/types.hpp"

using namespace proof_system;

namespace proof_system::plonk {
namespace stdlib {

using namespace barretenberg;
using namespace plookup;

/**
 * Add two curve points in one of the following ways:
 *  one: p1 + p2
 *  lambda: p1 + λ.p2
 *  one_plus_lambda: p1 + (1 + λ).p2
 */
template <typename C>
point<C> pedersen_plookup_hash<C>::add_points(const point& p1, const point& p2, const AddType add_type)
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
        field_t y_3 = lambda.madd(p1.x - x_3, -p1.y);
        return point{ x_3, y_3 };
    }

    point p3{ witness_t(ctx, x_3_raw), witness_t(ctx, y_3_raw) };

    ctx->create_ecc_add_gate({ p1.x.witness_index,
                               p1.y.witness_index,
                               p2.x.witness_index,
                               p2.y.witness_index,
                               p3.x.witness_index,
                               p3.y.witness_index,
                               endomorphism_coefficient,
                               sign_coefficient });

    return p3;
}

/**
 * Hash a single field element using lookup tables.
 */
template <typename C>
point<C> pedersen_plookup_hash<C>::hash_single(const field_t& scalar, const bool parity, const bool skip_range_check)
{
    if (scalar.is_constant()) {
        C* ctx = scalar.get_context();
        const auto hash_native = crypto::pedersen_hash::lookup::hash_single(scalar.get_value(), parity).normalize();
        return { field_t(ctx, hash_native.x), field_t(ctx, hash_native.y) };
    }

    // Slice the input scalar in lower 126 and higher 128 bits.
    C* ctx = scalar.get_context();
    const field_t y_hi = witness_t(ctx, uint256_t(scalar.get_value()).slice(126, 256));
    const field_t y_lo = witness_t(ctx, uint256_t(scalar.get_value()).slice(0, 126));

    ReadData<field_t> lookup_hi, lookup_lo;

    // If `skip_range_check = true`, this implies the input scalar is 252 bits maximum.
    // i.e. we do not require a check that scalar slice sums < p .
    // We can also likely use a multitable with 1 less lookup
    if (parity) {
        lookup_lo = plookup_read<C>::get_lookup_accumulators(MultiTableId::PEDERSEN_RIGHT_LO, y_lo);
        lookup_hi = plookup_read<C>::get_lookup_accumulators(MultiTableId::PEDERSEN_RIGHT_HI, y_hi);
    } else {
        lookup_lo = plookup_read<C>::get_lookup_accumulators(MultiTableId::PEDERSEN_LEFT_LO, y_lo);
        lookup_hi = plookup_read<C>::get_lookup_accumulators(MultiTableId::PEDERSEN_LEFT_HI, y_hi);
    }

    // validate slices equal scalar
    // TODO(suyash?): can remove this gate if we use a single lookup accumulator for HI + LO combined
    //       can recover y_hi, y_lo from Column 1 of the the lookup accumulator output
    scalar.add_two(-y_hi * (uint256_t(1) << 126), -y_lo).assert_equal(0);

    // if skip_range_check = true we assume input max size is 252 bits => final lookup scalar slice value must be 0
    if (skip_range_check) {
        lookup_hi[ColumnIdx::C1][lookup_hi[ColumnIdx::C1].size() - 1].assert_equal(0);
    }
    if (!skip_range_check) {
        // Check that y_hi * 2^126 + y_lo < fr::modulus when evaluated over the integers
        constexpr uint256_t modulus = fr::modulus;
        const field_t r_lo = field_t(ctx, modulus.slice(0, 126));
        const field_t r_hi = field_t(ctx, modulus.slice(126, 256));

        bool need_borrow = (uint256_t(y_lo.get_value()) > uint256_t(r_lo.get_value()));
        field_t borrow = field_t::from_witness(ctx, need_borrow);

        // directly call `create_new_range_constraint` to avoid creating an arithmetic gate
        scalar.get_context()->create_new_range_constraint(borrow.get_witness_index(), 1, "borrow");

        // Hi range check = r_hi - y_hi - borrow
        // Lo range check = r_lo - y_lo + borrow * 2^{126}
        field_t hi = (r_hi - y_hi) - borrow;
        field_t lo = (r_lo - y_lo) + (borrow * (uint256_t(1) << 126));

        hi.create_range_constraint(128);
        lo.create_range_constraint(126);
    }
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

/**
 * Hash a bunch of field element using merkle damagard construction.
 */
template <typename C>
field_t<C> pedersen_plookup_hash<C>::hash_multiple(const std::vector<field_t>& inputs, const size_t hash_index)
{
    if (inputs.size() == 0) {
        return point{ 0, 0 }.x;
    }

    auto result = plookup_read<C>::get_lookup_accumulators(MultiTableId::PEDERSEN_IV, hash_index)[ColumnIdx::C2][0];
    auto num_inputs = inputs.size();
    for (size_t i = 0; i < num_inputs; i++) {
        auto p2 = pedersen_plookup_hash<C>::hash_single(result, false);
        auto p1 = pedersen_plookup_hash<C>::hash_single(inputs[i], true);
        result = add_points(p1, p2).x;
    }

    auto p2 = hash_single(result, false);
    auto p1 = hash_single(field_t(num_inputs), true);
    return add_points(p1, p2).x;
}

INSTANTIATE_STDLIB_ULTRA_TYPE(pedersen_plookup_hash);

} // namespace stdlib
} // namespace proof_system::plonk
