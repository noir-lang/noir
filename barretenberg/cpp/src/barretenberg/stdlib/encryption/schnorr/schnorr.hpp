#pragma once
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/group/cycle_group.hpp"
#include "../../primitives/witness/witness.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"

namespace bb::plonk::stdlib::schnorr {

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

} // namespace bb::plonk::stdlib::schnorr
