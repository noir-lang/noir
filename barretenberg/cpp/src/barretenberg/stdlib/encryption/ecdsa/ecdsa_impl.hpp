#pragma once

#include "barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "barretenberg/stdlib/primitives//bit_array/bit_array.hpp"

namespace bb::stdlib {

/**
 * @brief Verify ECDSA signature. Produces unsatisfiable constraints if signature fails
 *
 * @tparam Builder
 * @tparam Curve
 * @tparam Fq
 * @tparam Fr
 * @tparam G1
 * @param message
 * @param public_key
 * @param sig
 * @return bool_t<Builder>
 */
template <typename Builder, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Builder> ecdsa_verify_signature(const stdlib::byte_array<Builder>& message,
                                       const G1& public_key,
                                       const ecdsa_signature<Builder>& sig)
{
    Builder* ctx = message.get_context() ? message.get_context() : public_key.x.context;

    /**
     * Check if recovery id v is either 27 ot 28.
     *
     * The v in an (r, s, v) ecdsa signature is the 8-bit recovery id s.t. v ∈ {0, 1, 2, 3}.
     * It is used to recover signing public key from an ecdsa signature. In practice, the value
     * of v is offset by 27 following the convention from the original bitcoin whitepaper.
     *
     * The value of v depends on the the point R = (x, y) s.t. r = x % |Fr|
     * 0: y is even  &&  x < |Fr| (x = r)
     * 1: y is odd   &&  x < |Fr| (x = r)
     * 2: y is even  &&  |Fr| <= x < |Fq| (x = r + |Fr|)
     * 3: y is odd   &&  |Fr| <= x < |Fq| (x = r + |Fr|)
     *
     * It is highly unlikely for x be be in [|Fr|, |Fq|) for the secp256k1 curve because:
     * P(|Fr| <= x < |Fq|) = 1 - |Fr|/|Fq| ≈ 0.
     * Therefore, it is reasonable to assume that the value of v will always be 0 or 1
     * (i.e. 27 or 28 with offset). In fact, the ethereum yellow paper [1] only allows v to be 27 or 28
     * and considers signatures with v ∈ {29, 30} to be non-standard.
     *
     * TODO(Suyash): EIP-155 allows v > 35 to ensure different v on different chains.
     * Do we need to consider that in our circuits?
     *
     * References:
     * [1] Ethereum yellow paper, Appendix E: https://ethereum.github.io/yellowpaper/paper.pdf
     * [2] EIP-155: https://eips.ethereum.org/EIPS/eip-155
     *
     */
    // Note: This check is also present in the _noassert variation of this method.
    field_t<Builder>(sig.v).assert_is_in_set({ field_t<Builder>(27), field_t<Builder>(28) },
                                             "signature is non-standard");

    stdlib::byte_array<Builder> hashed_message =
        static_cast<stdlib::byte_array<Builder>>(stdlib::sha256<Builder>(message));

    Fr z(hashed_message);
    z.assert_is_in_field();

    Fr r(sig.r);
    // force r to be < secp256k1 group modulus, so we can compare with `result_mod_r` below
    r.assert_is_in_field();

    Fr s(sig.s);

    // r and s should not be zero
    r.assert_is_not_equal(Fr::zero());
    s.assert_is_not_equal(Fr::zero());

    // s should be less than |Fr| / 2
    // Read more about this at: https://www.derpturkey.com/inherent-malleability-of-ecdsa-signatures/amp/
    s.assert_less_than((Fr::modulus + 1) / 2);

    Fr u1 = z / s;
    Fr u2 = r / s;

    public_key.validate_on_curve();

    G1 result;
    // TODO(Cody): Having Plookup should not determine which curve is used.
    // Use special plookup secp256k1 ECDSA mul if available (this relies on k1 endomorphism, and cannot be used for
    // other curves)
    if constexpr (HasPlookup<Builder> && Curve::type == bb::CurveType::SECP256K1) {
        result = G1::secp256k1_ecdsa_mul(public_key, u1, u2);
    } else {
        result = G1::batch_mul({ G1::one(ctx), public_key }, { u1, u2 });
    }
    result.x.self_reduce();

    // transfer Fq value x to an Fr element and reduce mod r
    Fr result_mod_r(ctx, 0);
    result_mod_r.binary_basis_limbs[0].element = result.x.binary_basis_limbs[0].element;
    result_mod_r.binary_basis_limbs[1].element = result.x.binary_basis_limbs[1].element;
    result_mod_r.binary_basis_limbs[2].element = result.x.binary_basis_limbs[2].element;
    result_mod_r.binary_basis_limbs[3].element = result.x.binary_basis_limbs[3].element;
    result_mod_r.binary_basis_limbs[0].maximum_value = result.x.binary_basis_limbs[0].maximum_value;
    result_mod_r.binary_basis_limbs[1].maximum_value = result.x.binary_basis_limbs[1].maximum_value;
    result_mod_r.binary_basis_limbs[2].maximum_value = result.x.binary_basis_limbs[2].maximum_value;
    result_mod_r.binary_basis_limbs[3].maximum_value = result.x.binary_basis_limbs[3].maximum_value;

    result_mod_r.prime_basis_limb = result.x.prime_basis_limb;

    result_mod_r.assert_is_in_field();

    result_mod_r.binary_basis_limbs[0].element.assert_equal(r.binary_basis_limbs[0].element);
    result_mod_r.binary_basis_limbs[1].element.assert_equal(r.binary_basis_limbs[1].element);
    result_mod_r.binary_basis_limbs[2].element.assert_equal(r.binary_basis_limbs[2].element);
    result_mod_r.binary_basis_limbs[3].element.assert_equal(r.binary_basis_limbs[3].element);
    result_mod_r.prime_basis_limb.assert_equal(r.prime_basis_limb);
    return bool_t<Builder>(ctx, true);
}

/**
 * @brief Verify ECDSA signature. Returns 0 if signature fails (i.e. does not produce unsatisfiable constraints)
 *
 * @tparam Builder
 * @tparam Curve
 * @tparam Fq
 * @tparam Fr
 * @tparam G1
 * @param hashed_message
 * @param public_key
 * @param sig
 * @return bool_t<Builder>
 */
template <typename Builder, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Builder> ecdsa_verify_signature_prehashed_message_noassert(const stdlib::byte_array<Builder>& hashed_message,
                                                                  const G1& public_key,
                                                                  const ecdsa_signature<Builder>& sig)
{
    Builder* ctx = hashed_message.get_context() ? hashed_message.get_context() : public_key.x.context;

    Fr z(hashed_message);
    z.assert_is_in_field();

    Fr r(sig.r);
    // force r to be < secp256k1 group modulus, so we can compare with `result_mod_r` below
    r.assert_is_in_field();

    Fr s(sig.s);

    // r and s should not be zero
    r.assert_is_not_equal(Fr::zero());
    s.assert_is_not_equal(Fr::zero());

    // s should be less than |Fr| / 2
    // Read more about this at: https://www.derpturkey.com/inherent-malleability-of-ecdsa-signatures/amp/
    s.assert_less_than((Fr::modulus + 1) / 2);

    Fr u1 = z / s;
    Fr u2 = r / s;

    public_key.validate_on_curve();

    G1 result;
    // Use special plookup secp256k1 ECDSA mul if available (this relies on k1 endomorphism, and cannot be used for
    // other curves)
    if constexpr (HasPlookup<Builder> && Curve::type == bb::CurveType::SECP256K1) {
        result = G1::secp256k1_ecdsa_mul(public_key, u1, u2);
    } else {
        result = G1::batch_mul({ G1::one(ctx), public_key }, { u1, u2 });
    }
    result.x.self_reduce();

    // transfer Fq value x to an Fr element and reduce mod r
    Fr result_mod_r(ctx, 0);
    result_mod_r.binary_basis_limbs[0].element = result.x.binary_basis_limbs[0].element;
    result_mod_r.binary_basis_limbs[1].element = result.x.binary_basis_limbs[1].element;
    result_mod_r.binary_basis_limbs[2].element = result.x.binary_basis_limbs[2].element;
    result_mod_r.binary_basis_limbs[3].element = result.x.binary_basis_limbs[3].element;
    result_mod_r.binary_basis_limbs[0].maximum_value = result.x.binary_basis_limbs[0].maximum_value;
    result_mod_r.binary_basis_limbs[1].maximum_value = result.x.binary_basis_limbs[1].maximum_value;
    result_mod_r.binary_basis_limbs[2].maximum_value = result.x.binary_basis_limbs[2].maximum_value;
    result_mod_r.binary_basis_limbs[3].maximum_value = result.x.binary_basis_limbs[3].maximum_value;

    result_mod_r.prime_basis_limb = result.x.prime_basis_limb;

    result_mod_r.assert_is_in_field();

    bool_t<Builder> output(ctx, true);
    output &= result_mod_r.binary_basis_limbs[0].element == (r.binary_basis_limbs[0].element);
    output &= result_mod_r.binary_basis_limbs[1].element == (r.binary_basis_limbs[1].element);
    output &= result_mod_r.binary_basis_limbs[2].element == (r.binary_basis_limbs[2].element);
    output &= result_mod_r.binary_basis_limbs[3].element == (r.binary_basis_limbs[3].element);
    output &= result_mod_r.prime_basis_limb == (r.prime_basis_limb);

    field_t<Builder>(sig.v).assert_is_in_set({ field_t<Builder>(27), field_t<Builder>(28) },
                                             "signature is non-standard");

    return output;
}

/**
 * @brief Verify ECDSA signature. Returns 0 if signature fails (i.e. does not produce unsatisfiable constraints)
 *
 * @tparam Builder
 * @tparam Curve
 * @tparam Fq
 * @tparam Fr
 * @tparam G1
 * @param message
 * @param public_key
 * @param sig
 * @return bool_t<Builder>
 */
template <typename Builder, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Builder> ecdsa_verify_signature_noassert(const stdlib::byte_array<Builder>& message,
                                                const G1& public_key,
                                                const ecdsa_signature<Builder>& sig)
{
    stdlib::byte_array<Builder> hashed_message =
        static_cast<stdlib::byte_array<Builder>>(stdlib::sha256<Builder>(message));

    return ecdsa_verify_signature_prehashed_message_noassert<Builder, Curve, Fq, Fr, G1>(
        hashed_message, public_key, sig);
}

} // namespace bb::stdlib