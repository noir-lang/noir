// #include "index.hpp"

// // #include "aztec3/constants.hpp"
//
// #include <gtest/gtest.h>
// #include <barretenberg/barretenberg.hpp>

// namespace aztec3::circuits::apps {

// namespace {
// using Builder = aztec3::utils::types:plonk::stdlib::types::Builder;
// using CT = aztec3::utils::types::CircuitTypes<Builder>;
// using NT = aztec3::utils::types::NativeTypes;
// // using plonk::stdlib::pedersen;
// } // namespace

// class private_state_tests : public ::testing::Test {};

// // TEST(private_state_tests, test_native_private_state)
// // {
// //     StateFactory<NT> state_factory("MyContract");
// //     PrivateStateVar<NT> x = state_factory.declare_private_state_var("x");

// //     PrivateStateVar<NT> native_private_state = PrivateStateVar(x);

// //     auto buffer = to_buffer(native_private_state);

// //     auto native_private_state_2 = from_buffer<PrivateStateVar<NT>>(buffer.data());

// //     EXPECT_EQ(native_private_state, native_private_state_2);
// // }

// // TEST(private_state_tests, test_create_private_state)
// // {
// //     StateFactory<NT> state_factory("MyContract");

// //     state_factory.declare_private_state_var("balances", { "asset_id", "owner" });

// //     state_factory.declare_private_state_var("x");

// //     // info("state_factory: ", state_factory);
// // }

// // TEST(private_state_tests, test_native_private_state_note_preimage)
// // {
// //     StateFactory<NT> state_factory("MyContract");
// //     PrivateStateVar<NT> x = state_factory.declare_private_state_var("x");

// //     PrivateStateNotePreimage<NT> native_preimage = {
// //         .value = 2,
// //         .owner = 3,
// //         .creator_address = NT::address(4),
// //         .salt = 5,
// //         .nonce = 6,
// //         .memo = 7,
// //     };

// //     auto buffer = to_buffer(native_preimage);

// //     auto native_preimage_2 = from_buffer<PrivateStateNotePreimage<NT>>(buffer.data());

// //     EXPECT_EQ(native_preimage, native_preimage_2);
// // }

// // TEST(private_state_tests, test_native_private_state_note_preimage_mapping)
// // {
// //     StateFactory<NT> state_factory("MyContract");
// //     PrivateStateVar<NT> x = state_factory.declare_private_state_var("x", { "mapping_key_name_1",
// "mapping_key_name_2"
// //     });

// //     PrivateStateNotePreimage<NT> native_preimage = {
// //         .mapping_key_values_by_key_name = std::map<std::string, std::optional<fr>>({ { "mapping_key_name_2", 5 }
// }),
// //         .value = 2,
// //         .owner = 3,
// //         .creator_address = NT::address(4),
// //         .salt = 5,
// //         .nonce = 6,
// //         .memo = 7,
// //     };

// //     auto buffer = to_buffer(native_preimage);

// //     auto native_preimage_2 = from_buffer<PrivateStateNotePreimage<NT>>(buffer.data());

// //     EXPECT_EQ(native_preimage, native_preimage_2);
// // }

// // TEST(private_state_tests, test_native_private_state_note_mapping)
// // {
// //     StateFactory<NT> state_factory("MyContract");
// //     PrivateStateVar<NT> x = state_factory.declare_private_state_var("x", { "mapping_key_name_1",
// "mapping_key_name_2"
// //     });

// //     PrivateStateNotePreimage<NT> private_state_preimage = {
// //         .mapping_key_values_by_key_name = std::map<std::string, std::optional<fr>>({ { "mapping_key_name_2", 5 }
// }),
// //         .value = 2,
// //         .owner = 3,
// //         .creator_address = NT::address(4),
// //         .salt = 5,
// //         .nonce = 6,
// //         .memo = 7,
// //     };

// //     PrivateStateNote<NT> private_state_note = PrivateStateNote<NT>(x, private_state_preimage);

// //     auto buffer = to_buffer(private_state_note);

// //     auto private_state_note_2 = from_buffer<PrivateStateNote<NT>>(buffer.data());

// //     EXPECT_EQ(private_state_note, private_state_note_2);
// // }

// /// TODO: figure out how to catch and test errors in gtest.

// } // namespace aztec3::circuits::apps