#pragma once
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/group/cycle_group.hpp"
#include "../../primitives/witness/witness.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"

namespace proof_system::plonk::stdlib::schnorr {

template <typename C> struct signature_bits {
    typename cycle_group<C>::cycle_scalar s;
    typename cycle_group<C>::cycle_scalar e;
};

template <typename C> signature_bits<C> convert_signature(C* context, const crypto::schnorr::signature& sig);

template <typename C>
std::array<field_t<C>, 2> verify_signature_internal(const byte_array<C>& message,
                                                    const cycle_group<C>& pub_key,
                                                    const signature_bits<C>& sig);

template <typename C>
void verify_signature(const byte_array<C>& message, const cycle_group<C>& pub_key, const signature_bits<C>& sig);

template <typename C>
bool_t<C> signature_verification_result(const byte_array<C>& message,
                                        const cycle_group<C>& pub_key,
                                        const signature_bits<C>& sig);

#define VERIFY_SIGNATURE_INTERNAL(circuit_type)                                                                        \
    std::array<field_t<circuit_type>, 2> verify_signature_internal<circuit_type>(                                      \
        const byte_array<circuit_type>&, const cycle_group<circuit_type>&, const signature_bits<circuit_type>&)

#define VERIFY_SIGNATURE(circuit_type)                                                                                 \
    void verify_signature<circuit_type>(                                                                               \
        const byte_array<circuit_type>&, const cycle_group<circuit_type>&, const signature_bits<circuit_type>&)

#define SIGNATURE_VERIFICATION_RESULT(circuit_type)                                                                    \
    bool_t<circuit_type> signature_verification_result<circuit_type>(                                                  \
        const byte_array<circuit_type>&, const cycle_group<circuit_type>&, const signature_bits<circuit_type>&)

#define CONVERT_SIGNATURE(circuit_type)                                                                                \
    signature_bits<circuit_type> convert_signature<circuit_type>(circuit_type*, const crypto::schnorr::signature&)

EXTERN_STDLIB_METHOD(VERIFY_SIGNATURE_INTERNAL)
EXTERN_STDLIB_METHOD(VERIFY_SIGNATURE)
EXTERN_STDLIB_METHOD(SIGNATURE_VERIFICATION_RESULT)
EXTERN_STDLIB_METHOD(CONVERT_SIGNATURE)

} // namespace proof_system::plonk::stdlib::schnorr
