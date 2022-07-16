// #include <gtest/gtest.h>
// #include <common/test.hpp>
// // #include <common/serialize.hpp>
// // #include <stdlib/types/turbo.hpp>
// // #include <aztec3/oracle/oracle.hpp>
// // #include <aztec3/circuits/apps/oracle_wrapper.hpp>
// // #include <numeric/random/engine.hpp>
// #include "index.hpp"
// // #include "deposit.hpp"
// // #include <aztec3/constants.hpp>
// // #include <crypto/pedersen/pedersen.hpp>
// // #include <stdlib/hash/pedersen/pedersen.hpp>

// namespace aztec3::circuits::apps::test_apps::private_to_private_function_call {

// class escrow_tests : public ::testing::Test {};

// TEST(escrow_tests, test_deposit)
// {

//     Composer composer;
//     DB db;

//     const NT::address contract_address = 12345;
//     const NT::address msg_sender =
//         NT::fr(uint256_t(0x01071e9a23e0f7edULL, 0x5d77b35d1830fa3eULL, 0xc6ba3660bb1f0c0bULL,
//         0x2ef9f7f09867fd6eULL));
//     const NT::fr msg_sender_private_key = 123456789;

//     NativeOracle oracle = NativeOracle(db, contract_address, msg_sender, msg_sender_private_key);
//     OracleWrapper oracle_wrapper = OracleWrapper(composer, oracle);

//     auto a = NT::fr(111);
//     auto b = NT::fr(222);
//     auto c = NT::fr(333);

//     auto result = function1(composer, oracle_wrapper, a, b, c);
//     info("result: ", result);

//     info("computed witness: ", composer.computed_witness);
//     info("witness: ", composer.witness);
//     // info("constant variables: ", composer.constant_variables);
//     // info("variables: ", composer.variables);
//     info("failed?: ", composer.failed);
//     info("err: ", composer.err);
//     info("n: ", composer.n);
// }

// } // namespace aztec3::circuits::apps::test_apps::private_to_private_function_call