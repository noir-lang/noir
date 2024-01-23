#pragma once
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/group/cycle_group.hpp"
#include "../../primitives/witness/witness.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"

namespace bb::stdlib {

template <typename C> struct schnorr_signature_bits {
    typename cycle_group<C>::cycle_scalar s;
    typename cycle_group<C>::cycle_scalar e;
};

template <typename C>
schnorr_signature_bits<C> schnorr_convert_signature(C* context, const crypto::schnorr_signature& sig);

template <typename C>
std::array<field_t<C>, 2> schnorr_verify_signature_internal(const byte_array<C>& message,
                                                            const cycle_group<C>& pub_key,
                                                            const schnorr_signature_bits<C>& sig);

template <typename C>
void schnorr_verify_signature(const byte_array<C>& message,
                              const cycle_group<C>& pub_key,
                              const schnorr_signature_bits<C>& sig);

template <typename C>
bool_t<C> schnorr_signature_verification_result(const byte_array<C>& message,
                                                const cycle_group<C>& pub_key,
                                                const schnorr_signature_bits<C>& sig);

} // namespace bb::stdlib
