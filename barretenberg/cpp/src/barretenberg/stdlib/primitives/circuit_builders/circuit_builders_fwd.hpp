/**
 * @brief Defines particular circuit builder types expected to be used for circuit
construction in stdlib and contains macros for explicit instantiation.
 *
 * @details This file is designed to be included in header files to instruct the compiler that these classes exist and
 * their instantiation will eventually take place. Given it has no dependencies, it causes no additional compilation or
 *  propagation.
 */
#pragma once
#include <concepts>

namespace bb::honk::flavor {
class Standard;
class Ultra;
} // namespace bb::honk::flavor

namespace bb {
class Bn254FrParams;
class Bn254FqParams;
template <class Params> struct alignas(32) field;
} // namespace bb
namespace arithmetization {
template <typename FF_> class Ultra;
} // namespace arithmetization
namespace bb {
template <class FF> class StandardCircuitBuilder_;
using StandardCircuitBuilder = StandardCircuitBuilder_<bb::field<bb::Bn254FrParams>>;
using StandardGrumpkinCircuitBuilder = StandardCircuitBuilder_<bb::field<bb::Bn254FqParams>>;
template <class Arithmetization> class UltraCircuitBuilder_;
using UltraCircuitBuilder = UltraCircuitBuilder_<arithmetization::Ultra<bb::field<bb::Bn254FrParams>>>;
template <class FF> class GoblinUltraCircuitBuilder_;
using GoblinUltraCircuitBuilder = GoblinUltraCircuitBuilder_<bb::field<bb::Bn254FrParams>>;
} // namespace bb