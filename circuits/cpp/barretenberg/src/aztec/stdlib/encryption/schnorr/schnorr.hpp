#pragma once
#include <crypto/schnorr/schnorr.hpp>
#include "../../primitives/field/field.hpp"
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/witness/witness.hpp"
#include "../../primitives/bit_array/bit_array.hpp"
#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/point/point.hpp"
#include "../../primitives/group/group.hpp"

namespace plonk {
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

template <typename C> point<C> variable_base_mul(const point<C>& pub_key, const bit_array<C>& scalar);
template <typename C> point<C> variable_base_mul(const point<C>& pub_key, const point<C>& current_accumulator, const wnaf_record<C>& scalar);
template <typename C> point<C> variable_base_mul(const point<C>& pub_key, const field_t<C>& low_bits, const field_t<C>& high_bits);


template <typename C> signature_bits<C> convert_signature(C* context, const crypto::schnorr::signature& sig);

template <typename C> bit_array<C> convert_message(C* context, const std::string& message_string);

template <typename C>
bool verify_signature(const byte_array<C>& message, const point<C>& pub_key, const signature_bits<C>& sig);

extern template point<waffle::TurboComposer> variable_base_mul<waffle::TurboComposer>(
    const point<waffle::TurboComposer>&, const point<waffle::TurboComposer>&, const wnaf_record<waffle::TurboComposer>&);

extern template point<waffle::TurboComposer> variable_base_mul(const point<waffle::TurboComposer>& pub_key, const field_t<waffle::TurboComposer>& low_bits, const field_t<waffle::TurboComposer>& high_bits);

extern template point<waffle::TurboComposer> variable_base_mul<waffle::TurboComposer>(
    const point<waffle::TurboComposer>&, const bit_array<waffle::TurboComposer>&);
extern template point<waffle::PlookupComposer> variable_base_mul<waffle::PlookupComposer>(
    const point<waffle::PlookupComposer>&, const bit_array<waffle::PlookupComposer>&);

extern template wnaf_record<waffle::TurboComposer> convert_field_into_wnaf<waffle::TurboComposer>(waffle::TurboComposer* context, const field_t<waffle::TurboComposer>& limb);

extern template bool verify_signature<waffle::TurboComposer>(const byte_array<waffle::TurboComposer>&,
                                                             const point<waffle::TurboComposer>&,
                                                             const signature_bits<waffle::TurboComposer>&);
extern template bool verify_signature<waffle::PlookupComposer>(const byte_array<waffle::PlookupComposer>&,
                                                               const point<waffle::PlookupComposer>&,
                                                               const signature_bits<waffle::PlookupComposer>&);

extern template signature_bits<waffle::TurboComposer> convert_signature<waffle::TurboComposer>(
    waffle::TurboComposer*, const crypto::schnorr::signature&);
extern template signature_bits<waffle::PlookupComposer> convert_signature<waffle::PlookupComposer>(
    waffle::PlookupComposer*, const crypto::schnorr::signature&);

extern template bit_array<waffle::TurboComposer> convert_message<waffle::TurboComposer>(waffle::TurboComposer*,
                                                                                        const std::string&);
extern template bit_array<waffle::PlookupComposer> convert_message<waffle::PlookupComposer>(waffle::PlookupComposer*,
                                                                                            const std::string&);

} // namespace schnorr
} // namespace stdlib
} // namespace plonk
