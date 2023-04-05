#pragma once
#include "barretenberg/crypto/schnorr/schnorr.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/witness/witness.hpp"
#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/point/point.hpp"
#include "../../primitives/group/group.hpp"

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

extern template point<plonk::TurboComposer> variable_base_mul<plonk::TurboComposer>(
    const point<plonk::TurboComposer>&, const point<plonk::TurboComposer>&, const wnaf_record<plonk::TurboComposer>&);

extern template point<plonk::TurboComposer> variable_base_mul(const point<plonk::TurboComposer>& pub_key,
                                                              const field_t<plonk::TurboComposer>& low_bits,
                                                              const field_t<plonk::TurboComposer>& high_bits);

extern template wnaf_record<plonk::TurboComposer> convert_field_into_wnaf<plonk::TurboComposer>(
    plonk::TurboComposer* context, const field_t<plonk::TurboComposer>& limb);

extern template wnaf_record<plonk::UltraComposer> convert_field_into_wnaf<plonk::UltraComposer>(
    plonk::UltraComposer* context, const field_t<plonk::UltraComposer>& limb);

extern template std::array<field_t<plonk::TurboComposer>, 2> verify_signature_internal<plonk::TurboComposer>(
    const byte_array<plonk::TurboComposer>&,
    const point<plonk::TurboComposer>&,
    const signature_bits<plonk::TurboComposer>&);
extern template std::array<field_t<plonk::UltraComposer>, 2> verify_signature_internal<plonk::UltraComposer>(
    const byte_array<plonk::UltraComposer>&,
    const point<plonk::UltraComposer>&,
    const signature_bits<plonk::UltraComposer>&);

extern template void verify_signature<plonk::TurboComposer>(const byte_array<plonk::TurboComposer>&,
                                                            const point<plonk::TurboComposer>&,
                                                            const signature_bits<plonk::TurboComposer>&);
extern template void verify_signature<plonk::UltraComposer>(const byte_array<plonk::UltraComposer>&,
                                                            const point<plonk::UltraComposer>&,
                                                            const signature_bits<plonk::UltraComposer>&);

extern template bool_t<plonk::TurboComposer> signature_verification_result<plonk::TurboComposer>(
    const byte_array<plonk::TurboComposer>&,
    const point<plonk::TurboComposer>&,
    const signature_bits<plonk::TurboComposer>&);
extern template bool_t<plonk::UltraComposer> signature_verification_result<plonk::UltraComposer>(
    const byte_array<plonk::UltraComposer>&,
    const point<plonk::UltraComposer>&,
    const signature_bits<plonk::UltraComposer>&);

extern template signature_bits<plonk::TurboComposer> convert_signature<plonk::TurboComposer>(
    plonk::TurboComposer*, const crypto::schnorr::signature&);
extern template signature_bits<plonk::UltraComposer> convert_signature<plonk::UltraComposer>(
    plonk::UltraComposer*, const crypto::schnorr::signature&);

} // namespace schnorr
} // namespace stdlib
} // namespace proof_system::plonk
