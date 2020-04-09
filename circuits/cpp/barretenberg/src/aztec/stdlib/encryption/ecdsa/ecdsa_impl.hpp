#pragma once

#include "../../hash/sha256/sha256.hpp"
#include "../../primitives/bit_array/bit_array.hpp"
namespace plonk {
namespace stdlib {
namespace ecdsa {

template <typename Composer, typename Fq, typename Fr, typename G1>
bool_t<Composer> verify_signature(const stdlib::byte_array<Composer>& message,
                                  const G1& public_key,
                                  const signature<Composer>& sig)
{
    Composer* ctx = message.get_context() ? message.get_context() : public_key.x.context;

    stdlib::bit_array<Composer> message_schedule(message);

    stdlib::byte_array<Composer> hashed_message =
        static_cast<stdlib::byte_array<Composer>>(stdlib::sha256<Composer>(message_schedule));

    Fr z(hashed_message);
    z.assert_is_in_field();

    Fr r(sig.r);
    Fr s(sig.s);

    Fr u1 = z / s;
    Fr u2 = r / s;

    G1 result = G1::batch_mul({ G1::one(ctx), public_key }, { u1, u2 });
    result.x.assert_is_in_field();

    field_t<Composer> result_x_lo =
        result.x.binary_basis_limbs[1].element * Fq::shift_1 + result.x.binary_basis_limbs[0].element;
    field_t<Composer> result_x_hi =
        result.x.binary_basis_limbs[3].element * Fq::shift_1 + result.x.binary_basis_limbs[2].element;

    Fr result_mod_r(result_x_lo, result_x_hi);
    result_mod_r.assert_is_in_field();
    r.assert_is_in_field();

    ctx->assert_equal(result_mod_r.binary_basis_limbs[0].element.witness_index,
                      r.binary_basis_limbs[0].element.witness_index);
    ctx->assert_equal(result_mod_r.binary_basis_limbs[1].element.witness_index,
                      r.binary_basis_limbs[1].element.witness_index);
    ctx->assert_equal(result_mod_r.binary_basis_limbs[2].element.witness_index,
                      r.binary_basis_limbs[2].element.witness_index);
    ctx->assert_equal(result_mod_r.binary_basis_limbs[3].element.witness_index,
                      r.binary_basis_limbs[3].element.witness_index);
    ctx->assert_equal(result_mod_r.prime_basis_limb.witness_index, r.prime_basis_limb.witness_index);

    return bool_t<Composer>(ctx, true);
}

} // namespace ecdsa
} // namespace stdlib
} // namespace plonk