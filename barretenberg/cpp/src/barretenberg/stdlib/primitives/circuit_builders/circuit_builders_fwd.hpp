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

namespace proof_system::honk {
namespace flavor {
class Standard;
class Ultra;
} // namespace flavor
} // namespace proof_system::honk

namespace barretenberg {
class Bn254FrParams;
class Bn254FqParams;
template <class Params> struct alignas(32) field;
} // namespace barretenberg
namespace arithmetization {
template <typename FF_> class Ultra;
} // namespace arithmetization
namespace proof_system {
template <class FF> class StandardCircuitBuilder_;
using StandardCircuitBuilder = StandardCircuitBuilder_<barretenberg::field<barretenberg::Bn254FrParams>>;
using StandardGrumpkinCircuitBuilder = StandardCircuitBuilder_<barretenberg::field<barretenberg::Bn254FqParams>>;
template <class Arithmetization> class UltraCircuitBuilder_;
using UltraCircuitBuilder =
    UltraCircuitBuilder_<arithmetization::Ultra<barretenberg::field<barretenberg::Bn254FrParams>>>;
template <class FF> class GoblinUltraCircuitBuilder_;
using GoblinUltraCircuitBuilder = GoblinUltraCircuitBuilder_<barretenberg::field<barretenberg::Bn254FrParams>>;
} // namespace proof_system