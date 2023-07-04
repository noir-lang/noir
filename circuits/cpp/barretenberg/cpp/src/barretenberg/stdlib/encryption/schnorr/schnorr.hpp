#pragma once
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/group/group.hpp"
#include "../../primitives/point/point.hpp"
#include "../../primitives/witness/witness.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"

namespace proof_system::plonk {
namespace stdlib {
namespace schnorr {

template <typename C> struct signature_bits {
    field_t<C> s_lo;
    field_t<C> s_hi;
    field_t<C> e_lo;
    field_t<C> e_hi;
};

template <typename C> struct wnaf_record {
    std::vector<bool_t<C>> bits;
    bool_t<C> skew;
};

template <typename C> wnaf_record<C> convert_field_into_wnaf(C* context, const field_t<C>& limb);

template <typename C>
point<C> variable_base_mul(const point<C>& pub_key, const point<C>& current_accumulator, const wnaf_record<C>& scalar);
template <typename C>
point<C> variable_base_mul(const point<C>& pub_key, const field_t<C>& low_bits, const field_t<C>& high_bits);

template <typename C> signature_bits<C> convert_signature(C* context, const crypto::schnorr::signature& sig);

template <typename C>
std::array<field_t<C>, 2> verify_signature_internal(const byte_array<C>& message,
                                                    const point<C>& pub_key,
                                                    const signature_bits<C>& sig);

template <typename C>
void verify_signature(const byte_array<C>& message, const point<C>& pub_key, const signature_bits<C>& sig);

template <typename C>
bool_t<C> signature_verification_result(const byte_array<C>& message,
                                        const point<C>& pub_key,
                                        const signature_bits<C>& sig);

#define VARIABLE_BASE_MUL(circuit_type)                                                                                \
    point<circuit_type> variable_base_mul<circuit_type>(                                                               \
        const point<circuit_type>&, const point<circuit_type>&, const wnaf_record<circuit_type>&)

#define CONVERT_FIELD_INTO_WNAF(circuit_type)                                                                          \
    wnaf_record<circuit_type> convert_field_into_wnaf<circuit_type>(circuit_type * context,                            \
                                                                    const field_t<circuit_type>& limb)

#define VERIFY_SIGNATURE_INTERNAL(circuit_type)                                                                        \
    std::array<field_t<circuit_type>, 2> verify_signature_internal<circuit_type>(                                      \
        const byte_array<circuit_type>&, const point<circuit_type>&, const signature_bits<circuit_type>&)

#define VERIFY_SIGNATURE(circuit_type)                                                                                 \
    void verify_signature<circuit_type>(                                                                               \
        const byte_array<circuit_type>&, const point<circuit_type>&, const signature_bits<circuit_type>&)

#define SIGNATURE_VERIFICATION_RESULT(circuit_type)                                                                    \
    bool_t<circuit_type> signature_verification_result<circuit_type>(                                                  \
        const byte_array<circuit_type>&, const point<circuit_type>&, const signature_bits<circuit_type>&)

#define CONVERT_SIGNATURE(circuit_type)                                                                                \
    signature_bits<circuit_type> convert_signature<circuit_type>(circuit_type*, const crypto::schnorr::signature&)

EXTERN_STDLIB_METHOD(VARIABLE_BASE_MUL)
EXTERN_STDLIB_METHOD(CONVERT_FIELD_INTO_WNAF)
EXTERN_STDLIB_METHOD(VERIFY_SIGNATURE_INTERNAL)
EXTERN_STDLIB_METHOD(VERIFY_SIGNATURE)
EXTERN_STDLIB_METHOD(SIGNATURE_VERIFICATION_RESULT)
EXTERN_STDLIB_METHOD(CONVERT_SIGNATURE)

} // namespace schnorr
} // namespace stdlib
} // namespace proof_system::plonk
