#pragma once
#include <stddef.h>
#include <stdint.h>
#include <numeric/uint256/uint256.hpp>
namespace rollup {

constexpr size_t DATA_TREE_DEPTH = 32;
constexpr size_t NULL_TREE_DEPTH = 256;
constexpr size_t ROOT_TREE_DEPTH = 28;
constexpr size_t DEFI_TREE_DEPTH = 30;

constexpr size_t MAX_NO_WRAP_INTEGER_BIT_LENGTH = 252;
constexpr size_t MAX_TXS_BIT_LENGTH = 10;
constexpr size_t TX_FEE_BIT_LENGTH = MAX_NO_WRAP_INTEGER_BIT_LENGTH - MAX_TXS_BIT_LENGTH;

constexpr size_t NUM_ASSETS_BIT_LENGTH = 4;
constexpr size_t NUM_ASSETS = 1 << NUM_ASSETS_BIT_LENGTH;
constexpr size_t ASSET_ID_BIT_LENGTH = 30;
constexpr size_t MAX_NUM_ASSETS_BIT_LENGTH = 30;
constexpr size_t MAX_NUM_ASSETS = 1 << MAX_NUM_ASSETS_BIT_LENGTH;
constexpr size_t ALIAS_HASH_BIT_LENGTH = 224;

constexpr uint32_t NUM_BRIDGE_CALLS_PER_BLOCK = 32;
constexpr uint32_t NUM_INTERACTION_RESULTS_PER_BLOCK = 32;

namespace circuit_gate_count {

/*
The boolean is_circuit_change_expected should be set to 0 by default. When there is an expected circuit change, the
developer can quickly check whether the circuit gate counts are in allowed range i.e., below the next power of two
limit, by setting it to one. However, while merging the corresponding PR, the developer should set
is_circuit_change_expected to zero and change the modified circuit gate counts accordingly.
*/
constexpr bool is_circuit_change_expected = 0;
/* The below constants are only used for regression testing; to identify accidental changes to circuit
 constraints. They need to be changed when there is a circuit change. */
constexpr uint32_t JOIN_SPLIT = 63984;
constexpr uint32_t ACCOUNT = 24123;
constexpr uint32_t CLAIM = 22684;
constexpr uint32_t ROLLUP = 1171925;
constexpr uint32_t ROOT_ROLLUP = 5477579;
constexpr uint32_t ROOT_VERIFIER = 7433260;
}; // namespace circuit_gate_count

namespace circuit_gate_next_power_of_two {
/* The below constants are used in tests to detect undesirable circuit changes. They should not be changed unless we
want to exceed the next power of two limit. */
constexpr uint32_t JOIN_SPLIT = 65536;
constexpr uint32_t ACCOUNT = 32768;
constexpr uint32_t CLAIM = 32768;
constexpr uint32_t ROLLUP = 2097152;
constexpr uint32_t ROOT_ROLLUP = 8388608;
constexpr uint32_t ROOT_VERIFIER = 8388608;
}; // namespace circuit_gate_next_power_of_two

namespace circuit_vk_hash {
/* These below constants are only used for regression testing; to identify accidental changes to circuit
 constraints. They need to be changed when there is a circuit change. Note that they are written in the reverse order
 to comply with the from_buffer<>() method. */
constexpr auto ACCOUNT = uint256_t(0x5d97584af28e524a, 0xd2007df20f73d2bf, 0xe134d5b853eff05a, 0xdd2c3d1ff8bce519);
constexpr auto JOIN_SPLIT = uint256_t(0x7d54acfcca0eaf34, 0xa69c7b4e1c2fb5c5, 0x77161b356ca1c7a2, 0xb88194f268d2caa9);
constexpr auto CLAIM = uint256_t(0x878301ebba40ab60, 0x931466762c62d661, 0x40aad71ec3496905, 0x9f47aaa109759d0a);
constexpr auto ROLLUP = uint256_t(0x3c9e491095547852, 0xbf65ec889ec96a1a, 0xb16e824aa0bb319f, 0x28d7b587edf1eb4d);
constexpr auto ROOT_ROLLUP = uint256_t(0xa5e06b55f0e30cbe, 0x5fbf39af52fe67c8, 0xd8a0ecd1bb3a6f40, 0xdf67f7fcbb55dc1f);
constexpr auto ROOT_VERIFIER =
    uint256_t(0x341a876aae2df472, 0x87e0704f1ae50773, 0x5d6c740f61a0dbdd, 0x1f1a94b50cdcf5ae);
}; // namespace circuit_vk_hash

namespace ProofIds {
enum { PADDING = 0, DEPOSIT = 1, WITHDRAW = 2, SEND = 3, ACCOUNT = 4, DEFI_DEPOSIT = 5, DEFI_CLAIM = 6 };
};

} // namespace rollup