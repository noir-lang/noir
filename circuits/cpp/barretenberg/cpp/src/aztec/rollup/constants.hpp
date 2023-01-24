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
constexpr uint32_t ACCOUNT = 23958;
constexpr uint32_t JOIN_SPLIT = 64000;
constexpr uint32_t CLAIM = 22684;
constexpr uint32_t ROLLUP = 1173221;
constexpr uint32_t ROOT_ROLLUP = 5481327;
constexpr uint32_t ROOT_VERIFIER = 7435892;
}; // namespace circuit_gate_count

namespace circuit_gate_next_power_of_two {
/* The below constants are used in tests to detect undesirable circuit changes. They should not be changed unless we
want to exceed the next power of two limit. */
constexpr uint32_t ACCOUNT = 32768;
constexpr uint32_t JOIN_SPLIT = 65536;
constexpr uint32_t CLAIM = 32768;
constexpr uint32_t ROLLUP = 2097152;
constexpr uint32_t ROOT_ROLLUP = 8388608;
constexpr uint32_t ROOT_VERIFIER = 8388608;
}; // namespace circuit_gate_next_power_of_two

namespace circuit_vk_hash {
/* These below constants are only used for regression testing; to identify accidental changes to circuit
 constraints. They need to be changed when there is a circuit change. Note that they are written in the reverse order
 to comply with the from_buffer<>() method. */
constexpr auto ACCOUNT = uint256_t(0x78ebf096ab73e440, 0xaa1dc7c26a125f6e, 0x488a97e465b96964, 0xf9d3e501b89bf466);
constexpr auto JOIN_SPLIT = uint256_t(0x5e67a4a4503ebf25, 0xb3c070c061e76d1a, 0xb18c6c6a5bcad5fb, 0xe0d5f46cafb33ecf);
constexpr auto CLAIM = uint256_t(0x878301ebba40ab60, 0x931466762c62d661, 0x40aad71ec3496905, 0x9f47aaa109759d0a);
constexpr auto ROLLUP = uint256_t(0x160731cc44173fdc, 0x6a6d55e46bf198bd, 0x9ce1d4608ae26fb0, 0x865ced5c16cb6152);
constexpr auto ROOT_ROLLUP = uint256_t(0xd77e82eae9e6efc7, 0x2b5ddf767012a4cf, 0x8b5982bb3d64616f, 0x20b515f5a9c78048);
constexpr auto ROOT_VERIFIER =
    uint256_t(0x8e8313d6015ca626, 0x62ccf70b81c4e099, 0x33bee0072a20f36a, 0x44bd24daa009cd59);
}; // namespace circuit_vk_hash

namespace ProofIds {
enum { PADDING = 0, DEPOSIT = 1, WITHDRAW = 2, SEND = 3, ACCOUNT = 4, DEFI_DEPOSIT = 5, DEFI_CLAIM = 6 };
};

} // namespace rollup