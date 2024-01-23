#include "schnorr.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/stdlib/hash/blake2s/blake2s.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"
#include <array>

namespace bb::stdlib {

/**
 * @brief Instantiate a witness containing the signature (s, e) as a quadruple of
 * field_t elements (s_lo, s_hi, e_lo, e_hi).
 */
template <typename C>
schnorr_signature_bits<C> schnorr_convert_signature(C* context, const crypto::schnorr_signature& signature)
{
    using cycle_scalar = typename cycle_group<C>::cycle_scalar;

    uint256_t s_bigint(0);
    uint256_t e_bigint(0);
    const uint8_t* s_ptr = &signature.s[0];
    const uint8_t* e_ptr = &signature.e[0];
    numeric::read(s_ptr, s_bigint);
    numeric::read(e_ptr, e_bigint);
    schnorr_signature_bits<C> sig{ .s = cycle_scalar::from_witness_bitstring(context, s_bigint, 256),
                                   .e = cycle_scalar::from_witness_bitstring(context, e_bigint, 256) };
    return sig;
}

/**
 * @brief Make the computations needed to verify a signature (s, e),  i.e., compute
 *          e' = hash(([s]g + [e]pub).x | message)
          and return e'.
 *
 * @details UltraPlonk: ~5018 gates, excluding gates required to init the UltraPlonk range check
 * (~1,169k for fixed/variable_base_mul, ~4k for blake2s) for a string of length = 34.
 */
template <typename C>
std::array<field_t<C>, 2> schnorr_verify_signature_internal(const byte_array<C>& message,
                                                            const cycle_group<C>& pub_key,
                                                            const schnorr_signature_bits<C>& sig)
{
    cycle_group<C> g1(grumpkin::g1::one);
    // compute g1 * sig.s + key * sig,e

    auto x_3 = cycle_group<C>::batch_mul({ sig.s, sig.e }, { g1, pub_key }).x;
    // build input (pedersen(([s]g + [e]pub).x | pub.x | pub.y) | message) to hash function
    // pedersen hash ([r].x | pub.x) to make sure the size of `hash_input` is <= 64 bytes for a 32 byte message
    byte_array<C> hash_input(pedersen_hash<C>::hash({ x_3, pub_key.x, pub_key.y }));
    hash_input.write(message);

    // compute  e' = hash(([s]g + [e]pub).x | message)
    byte_array<C> output = blake2s(hash_input);
    static constexpr size_t LO_BYTES = cycle_group<C>::cycle_scalar::LO_BITS / 8;
    static constexpr size_t HI_BYTES = 32 - LO_BYTES;
    field_t<C> output_hi(output.slice(0, LO_BYTES));
    field_t<C> output_lo(output.slice(LO_BYTES, HI_BYTES));
    return { output_lo, output_hi };
}

/**
 * @brief Verify that a signature (s, e) is valid, i.e., compute
 *          e' = hash(([s]g + [e]pub).x | message)
 *        and check that
 *          e' == e is true.
 */
template <typename C>
void schnorr_verify_signature(const byte_array<C>& message,
                              const cycle_group<C>& pub_key,
                              const schnorr_signature_bits<C>& sig)
{
    auto [output_lo, output_hi] = schnorr_verify_signature_internal(message, pub_key, sig);
    output_lo.assert_equal(sig.e.lo, "verify signature failed");
    output_hi.assert_equal(sig.e.hi, "verify signature failed");
}

/**
 * @brief Attempt to verify a signature (s, e) and return the result, i.e., compute
 *          e' = hash(([s]g + [e]pub).x | message)
 *        and return the boolean witness e' == e.
 */
template <typename C>
bool_t<C> schnorr_signature_verification_result(const byte_array<C>& message,
                                                const cycle_group<C>& pub_key,
                                                const schnorr_signature_bits<C>& sig)
{
    auto [output_lo, output_hi] = schnorr_verify_signature_internal(message, pub_key, sig);
    bool_t<C> valid = (output_lo == sig.e.lo) && (output_hi == sig.e.hi);
    return valid;
}

#define VERIFY_SIGNATURE_INTERNAL(circuit_type)                                                                        \
    template std::array<field_t<circuit_type>, 2> schnorr_verify_signature_internal<circuit_type>(                     \
        const byte_array<circuit_type>&,                                                                               \
        const cycle_group<circuit_type>&,                                                                              \
        const schnorr_signature_bits<circuit_type>&)
VERIFY_SIGNATURE_INTERNAL(bb::StandardCircuitBuilder);
VERIFY_SIGNATURE_INTERNAL(bb::UltraCircuitBuilder);
VERIFY_SIGNATURE_INTERNAL(bb::GoblinUltraCircuitBuilder);
#define VERIFY_SIGNATURE(circuit_type)                                                                                 \
    template void schnorr_verify_signature<circuit_type>(const byte_array<circuit_type>&,                              \
                                                         const cycle_group<circuit_type>&,                             \
                                                         const schnorr_signature_bits<circuit_type>&)
VERIFY_SIGNATURE(bb::StandardCircuitBuilder);
VERIFY_SIGNATURE(bb::UltraCircuitBuilder);
VERIFY_SIGNATURE(bb::GoblinUltraCircuitBuilder);
#define SIGNATURE_VERIFICATION_RESULT(circuit_type)                                                                    \
    template bool_t<circuit_type> schnorr_signature_verification_result<circuit_type>(                                 \
        const byte_array<circuit_type>&,                                                                               \
        const cycle_group<circuit_type>&,                                                                              \
        const schnorr_signature_bits<circuit_type>&)
SIGNATURE_VERIFICATION_RESULT(bb::StandardCircuitBuilder);
SIGNATURE_VERIFICATION_RESULT(bb::UltraCircuitBuilder);
SIGNATURE_VERIFICATION_RESULT(bb::GoblinUltraCircuitBuilder);
#define CONVERT_SIGNATURE(circuit_type)                                                                                \
    template schnorr_signature_bits<circuit_type> schnorr_convert_signature<circuit_type>(                             \
        circuit_type*, const crypto::schnorr_signature&)
CONVERT_SIGNATURE(bb::StandardCircuitBuilder);
CONVERT_SIGNATURE(bb::UltraCircuitBuilder);
CONVERT_SIGNATURE(bb::GoblinUltraCircuitBuilder);
} // namespace bb::stdlib
