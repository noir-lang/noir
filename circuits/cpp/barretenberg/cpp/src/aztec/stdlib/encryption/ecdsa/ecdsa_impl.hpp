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

    stdlib::byte_array<Composer> hashed_message =
        static_cast<stdlib::byte_array<Composer>>(stdlib::sha256<Composer>(message));

    Fr z(hashed_message);
    z.assert_is_in_field();

    Fr r(sig.r);
    Fr s(sig.s);

    // r and s should not be zero
    r.assert_is_not_equal(Fr::zero());
    s.assert_is_not_equal(Fr::zero());

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
    result_mod_r.binary_basis_limbs[0].element.assert_equal(r.binary_basis_limbs[0].element);
    result_mod_r.binary_basis_limbs[1].element.assert_equal(r.binary_basis_limbs[1].element);
    result_mod_r.binary_basis_limbs[2].element.assert_equal(r.binary_basis_limbs[2].element);
    result_mod_r.binary_basis_limbs[3].element.assert_equal(r.binary_basis_limbs[3].element);
    result_mod_r.prime_basis_limb.assert_equal(r.prime_basis_limb);

    return bool_t<Composer>(ctx, true);
}

} // namespace ecdsa
} // namespace stdlib
} // namespace plonk