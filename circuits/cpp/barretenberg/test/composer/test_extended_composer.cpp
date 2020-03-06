// #include <gtest/gtest.h>

// #include <barretenberg/waffle/composer/extended_composer.hpp>
// #include <barretenberg/waffle/proof_system/preprocess.hpp>
// #include <barretenberg/waffle/proof_system/prover/prover.hpp>
// #include <barretenberg/waffle/proof_system/verifier/verifier.hpp>
// #include <barretenberg/waffle/proof_system/widgets/arithmetic_widget.hpp>
// #include <barretenberg/waffle/proof_system/widgets/sequential_widget.hpp>

// #include <barretenberg/polynomials/polynomial_arithmetic.hpp>
// #include <barretenberg/waffle/stdlib/field/field.hpp>
// #include <barretenberg/waffle/stdlib/uint32/uint32.hpp>

// #include <memory>

// using namespace barretenberg;

// typedef plonk::stdlib::field_t<waffle::TurboComposer> field_t;
// typedef plonk::stdlib::uint32<waffle::TurboComposer> uint32;
// typedef plonk::stdlib::witness_t<waffle::TurboComposer> witness_t;

// namespace {
// uint32_t get_random_int()
// {
//     return static_cast<uint32_t>(barretenberg::fr::random_element().data[0]);
// }
// } // namespace

// TEST(extended_composer, test_combine_linear_relations_basic_add)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     fr wires[7]{ fr::one(), fr::one(), fr::one(), fr::one(), fr::one(), fr::one(), fr::one() };
//     uint32_t wire_indices[7]{ composer.add_variable(wires[0]), composer.add_variable(wires[1]),
//                               composer.add_variable(wires[2]), composer.add_variable(wires[3]),
//                               composer.add_variable(wires[4]), composer.add_variable(wires[5]),
//                               composer.add_variable(wires[6]) };
//     composer.create_add_gate(
//         { wire_indices[0], wire_indices[1], wire_indices[2], fr::one(), fr::one(), fr::neg_one(), fr::zero() });
//     composer.create_add_gate(
//         { wire_indices[2], wire_indices[3], wire_indices[4], fr::one(), fr::one(), fr::neg_one(), fr::zero() });
//     composer.create_add_gate(
//         { wire_indices[4], wire_indices[5], wire_indices[6], fr::one(), fr::one(), fr::neg_one(), fr::zero() });

//     composer.combine_linear_relations();

//     EXPECT_EQ(composer.is_gate_deleted(0), false);
//     EXPECT_EQ(composer.is_gate_deleted(1), true);
//     EXPECT_EQ(composer.is_gate_deleted(2), false);
//     EXPECT_EQ(composer.get_num_gates(), 2UL);
// }

// TEST(extended_composer, test_combine_linear_relations_basic_mul_add)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     fr wires[7]{ fr::one(), fr::one(), fr::one(), fr::one(), fr::one(), fr::one(), fr::one() };
//     uint32_t wire_indices[7]{ composer.add_variable(wires[0]), composer.add_variable(wires[1]),
//                               composer.add_variable(wires[2]), composer.add_variable(wires[3]),
//                               composer.add_variable(wires[4]), composer.add_variable(wires[5]),
//                               composer.add_variable(wires[6]) };
//     composer.create_mul_gate({ wire_indices[0], wire_indices[1], wire_indices[2], fr::one(), fr::neg_one(), fr::zero() });
//     composer.create_add_gate(
//         { wire_indices[2], wire_indices[3], wire_indices[4], fr::one(), fr::one(), fr::neg_one(), fr::zero() });
//     composer.create_add_gate(
//         { wire_indices[4], wire_indices[5], wire_indices[6], fr::one(), fr::one(), fr::neg_one(), fr::zero() });

//     composer.combine_linear_relations();

//     EXPECT_EQ(composer.is_gate_deleted(0), false);
//     EXPECT_EQ(composer.is_gate_deleted(1), true);
//     EXPECT_EQ(composer.is_gate_deleted(2), false);
//     EXPECT_EQ(composer.get_num_gates(), 2UL);
// }

// TEST(extended_composer, test_combine_linear_relations_uint32)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     uint32 a = witness_t(&composer, static_cast<uint32_t>(-1));
//     a.get_witness_index();
//     waffle::TurboProver prover = composer.preprocess();

//     EXPECT_EQ(composer.is_gate_deleted(0), false);
//     EXPECT_EQ(composer.is_gate_deleted(1), true);
//     EXPECT_EQ(composer.is_gate_deleted(2), false);
//     EXPECT_EQ(composer.is_gate_deleted(3), true);
//     EXPECT_EQ(composer.is_gate_deleted(4), false);
//     EXPECT_EQ(composer.is_gate_deleted(5), true);
//     EXPECT_EQ(composer.is_gate_deleted(6), false);
//     EXPECT_EQ(composer.is_gate_deleted(7), true);
//     EXPECT_EQ(composer.is_gate_deleted(8), false);
//     EXPECT_EQ(composer.is_gate_deleted(9), true);
//     EXPECT_EQ(composer.is_gate_deleted(10), false);
//     EXPECT_EQ(composer.is_gate_deleted(11), true);
//     EXPECT_EQ(composer.is_gate_deleted(12), false);
//     EXPECT_EQ(composer.is_gate_deleted(13), true);
//     EXPECT_EQ(composer.is_gate_deleted(14), false);
//     EXPECT_EQ(composer.is_gate_deleted(15), true);
//     EXPECT_EQ(composer.is_gate_deleted(16), false);
//     EXPECT_EQ(composer.is_gate_deleted(17), true);
//     EXPECT_EQ(composer.is_gate_deleted(18), false);
//     EXPECT_EQ(composer.is_gate_deleted(19), true);
//     EXPECT_EQ(composer.is_gate_deleted(20), false);
//     EXPECT_EQ(composer.is_gate_deleted(21), true);
//     EXPECT_EQ(composer.is_gate_deleted(22), false);
//     EXPECT_EQ(composer.is_gate_deleted(23), true);
//     EXPECT_EQ(composer.is_gate_deleted(24), false);
//     EXPECT_EQ(composer.is_gate_deleted(25), true);
//     EXPECT_EQ(composer.is_gate_deleted(26), false);
//     EXPECT_EQ(composer.is_gate_deleted(27), true);
//     EXPECT_EQ(composer.is_gate_deleted(28), false);
//     EXPECT_EQ(composer.is_gate_deleted(29), true);
//     EXPECT_EQ(composer.is_gate_deleted(30), false);

//     EXPECT_EQ(composer.q_1[0]).data[0], 1UL << 2UL.from_montgomery_form();
//     EXPECT_EQ(composer.q_2[0]).data[0], 1UL << 1UL.from_montgomery_form();
//     EXPECT_EQ(composer.q_3[0]).data[0], 1UL.from_montgomery_form();
//     EXPECT_EQ((composer.q_3_next[0] == fr::neg_one()), true);

//     for (size_t i = 2; i < 30; i += 2) {
//         uint64_t shift = static_cast<uint64_t>(i) + 1UL;
//         EXPECT_EQ(composer.q_1[i]).data[0], 1UL << (shift + 1UL).from_montgomery_form();
//         EXPECT_EQ(composer.q_2[i]).data[0], 1UL << shift.from_montgomery_form();
//         EXPECT_EQ((composer.q_3[i] == fr::one()), true);
//         EXPECT_EQ((composer.q_3_next[i] == fr::neg_one()), true);
//     }
//     EXPECT_EQ((composer.q_1[30] == fr::neg_one()), true);
//     EXPECT_EQ(composer.q_2[30]).data[0], 1UL << 31UL.from_montgomery_form();
//     EXPECT_EQ(composer.q_3[30]).data[0], 1UL.from_montgomery_form();
//     EXPECT_EQ((composer.q_3_next[30] == fr::zero()), true);

//     EXPECT_EQ(prover.witness->wires.at("w_1")[0]).data[0], 1UL.from_montgomery_form();
//     EXPECT_EQ(prover.witness->wires.at("w_2")[0]).data[0], 1UL.from_montgomery_form();
//     EXPECT_EQ(prover.witness->wires.at("w_3")[0]).data[0], 1UL.from_montgomery_form();
//     EXPECT_EQ(prover.witness->wires.at("w_3")[1]).data[0], (1UL << 3UL) - 1UL); // 7U.from_montgomery_form();
//     EXPECT_EQ(prover.witness->wires.at("w_3")[2]).data[0], (1UL << 5UL) - 1UL.from_montgomery_form();
//     EXPECT_EQ(prover.witness->wires.at("w_3")[3]).data[0], (1UL << 7U) - 1UL.from_montgomery_form();

//     for (size_t i = 1; i < 15; ++i) {
//         EXPECT_EQ(prover.witness->wires.at("w_1")[i]).data[0], 1UL.from_montgomery_form();
//         EXPECT_EQ(prover.witness->wires.at("w_2")[i]).data[0], 1UL.from_montgomery_form();
//         EXPECT_EQ(prover.witness->wires.at("w_3")[i]).data[0], (1U << static_cast<uint32_t>(2 * i + 1)) - 1.from_montgomery_form();
//     }

//     EXPECT_EQ(prover.witness->wires.at("w_2")[15]).data[0], 1UL.from_montgomery_form();
//     EXPECT_EQ(prover.witness->wires.at("w_1")[15]).data[0], (1ULL << 32ULL) - 1ULL.from_montgomery_form();
//     EXPECT_EQ(prover.witness->wires.at("w_3")[15]).data[0], (1ULL << 31ULL) - 1ULL.from_montgomery_form();

//     for (size_t i = 16; i < 32; ++i) {
//         EXPECT_EQ((prover.witness->wires.at("w_1")[i] == fr::zero()), true);
//         EXPECT_EQ((prover.witness->wires.at("w_2")[i] == fr::zero()), true);
//         EXPECT_EQ((prover.witness->wires.at("w_3")[i] == fr::zero()), true);
//     }
//     waffle::TurboVerifier verifier = composer.create_verifier();

//     waffle::plonk_proof proof = prover.construct_proof();

//     bool proof_valid = verifier.verify_proof(proof);

//     EXPECT_EQ(composer.get_num_gates(), 16UL);
//     EXPECT_EQ(proof_valid, true);
// }

// TEST(extended_composer, composer_consistency)
// {
//     waffle::StandardComposer standard_composer = waffle::StandardComposer();
//     waffle::TurboComposer extended_composer = waffle::TurboComposer();

//     plonk::stdlib::field_t<waffle::StandardComposer> a1[10];
//     plonk::stdlib::field_t<waffle::StandardComposer> b1[10];
//     plonk::stdlib::field_t<waffle::TurboComposer> a2[10];
//     plonk::stdlib::field_t<waffle::TurboComposer> b2[10];
//     for (size_t i = 0; i < 10; ++i) {
//         a1[i] = plonk::stdlib::witness_t<waffle::StandardComposer>(&standard_composer, 100U);
//         b1[i] = plonk::stdlib::witness_t<waffle::StandardComposer>(&standard_composer, 44U);
//         a2[i] = plonk::stdlib::witness_t<waffle::TurboComposer>(&extended_composer, 100U);
//         b2[i] = plonk::stdlib::witness_t<waffle::TurboComposer>(&extended_composer, 44U);
//         a1[i] * b1[i];
//         a2[i] * b2[i];
//     }

//     waffle::Prover standard_prover = standard_composer.preprocess();

//     waffle::TurboProver extended_prover = extended_composer.preprocess();

//     EXPECT_EQ(standard_prover.n, extended_prover.n);

//     for (size_t i = 0; i < standard_prover.n; ++i) {
//         EXPECT_EQ((standard_composer.q_m[i] == extended_composer.q_m[i]), true);
//         EXPECT_EQ((standard_composer.q_1[i] == extended_composer.q_1[i]), true);
//         EXPECT_EQ((standard_composer.q_2[i] == extended_composer.q_2[i]), true);
//         EXPECT_EQ((standard_composer.q_3[i] == extended_composer.q_3[i]), true);
//         EXPECT_EQ((standard_composer.q_c[i] == extended_composer.q_c[i]), true);
//         EXPECT_EQ((extended_composer.q_3_next[i] == fr::zero()), true);
//         EXPECT_EQ((extended_composer.q_left_bools[i] == fr::zero()), true);
//         EXPECT_EQ((extended_composer.q_right_bools[i] == fr::zero()), true);
//         EXPECT_EQ((standard_prover.witness->wires.at("w_1")[i] == extended_prover.witness->wires.at("w_1")[i]), true);
//         EXPECT_EQ((standard_prover.witness->wires.at("w_2")[i] == extended_prover.witness->wires.at("w_2")[i]), true);
//         EXPECT_EQ((standard_prover.witness->wires.at("w_3")[i] == extended_prover.witness->wires.at("w_3")[i]), true);
//         // EXPECT_EQ(standard_prover.sigma_1_mapping[i], extended_prover.sigma_1_mapping[i]);
//         // EXPECT_EQ(standard_prover.sigma_2_mapping[i], extended_prover.sigma_2_mapping[i]);
//         // EXPECT_EQ(standard_prover.sigma_3_mapping[i], extended_prover.sigma_3_mapping[i]);
//     }

//     waffle::Verifier verifier = standard_composer.create_verifier();

//     waffle::plonk_proof proof = standard_prover.construct_proof();

//     bool proof_valid = verifier.verify_proof(proof);

//     EXPECT_EQ(proof_valid, true);
// }

// TEST(extended_composer, basic_proof)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     field_t a[10];
//     field_t b[10];
//     for (size_t i = 0; i < 10; ++i) {
//         a[i] = witness_t(&composer, 100U);
//         b[i] = witness_t(&composer, 44U);
//         a[i] * b[i];
//     }

//     waffle::TurboProver prover = composer.preprocess();

//     EXPECT_EQ(composer.is_gate_deleted(0), false);
//     EXPECT_EQ(composer.is_gate_deleted(1), false);

//     waffle::TurboVerifier verifier = composer.create_verifier();

//     waffle::plonk_proof proof = prover.construct_proof();

//     bool proof_valid = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_valid, true);
// }

// TEST(extended_composer, basic_optimized_proof)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     uint32 a = witness_t(&composer, 100U);
//     uint32 b = witness_t(&composer, 44U);
//     // uint32 cc = witness_t(&composer, 44U);
//     uint32 c = a * b;
//     uint32 d = a * c;
//     c.get_witness_index();
//     d.get_witness_index();

//     waffle::TurboProver prover = composer.preprocess();

//     waffle::TurboVerifier verifier = composer.create_verifier();
//     waffle::plonk_proof proof = prover.construct_proof();

//     bool proof_valid = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_valid, true);
// }

// TEST(extended_composer, test_optimized_uint32_xor)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     uint32 a = witness_t(&composer, 100U);
//     uint32 b = witness_t(&composer, 44U);
//     uint32 c = a ^ b;
//     c = c + a;
//     waffle::TurboProver prover = composer.preprocess();

//     EXPECT_EQ(composer.get_num_gates(), 65UL);

//     waffle::TurboVerifier verifier = composer.create_verifier();
//     waffle::plonk_proof proof = prover.construct_proof();
//     bool proof_valid = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_valid, true);
// }

// TEST(extended_composer, test_optimized_uint32_and)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     uint32 a = witness_t(&composer, 100U);
//     uint32 b = witness_t(&composer, 44U);
//     uint32 c = a & b;
//     c = c + a;
//     waffle::TurboProver prover = composer.preprocess();

//     EXPECT_EQ(composer.get_num_gates(), 65UL);

//     waffle::TurboVerifier verifier = composer.create_verifier();
//     waffle::plonk_proof proof = prover.construct_proof();
//     bool proof_valid = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_valid, true);
// }

// TEST(extended_composer, test_optimized_uint32_or)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     uint32 a = witness_t(&composer, 100U);
//     uint32 b = witness_t(&composer, 44U);
//     uint32 c = a | b;
//     c = c + a;
//     waffle::TurboProver prover = composer.preprocess();

//     EXPECT_EQ(composer.get_num_gates(), 65UL);

//     waffle::TurboVerifier verifier = composer.create_verifier();
//     waffle::plonk_proof proof = prover.construct_proof();
//     bool proof_valid = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_valid, true);
// }

// TEST(extended_composer, small_optimized_circuit)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     std::array<uint32_t, 64> w_ref;
//     std::array<uint32, 64> w;
//     for (size_t i = 0; i < 64; ++i) {
//         w_ref[i] = get_random_int();
//         w[i] = witness_t(&composer, w_ref[i]);
//     }

//     for (size_t i = 16; i < 64; ++i) {
//         uint32 s0 = w[i - 15].ror(7) ^ w[i - 15].ror(18) ^ w[i - 15].ror(3);
//         uint32 s1 = w[i - 2].ror(17) ^ w[i - 2].ror(19) ^ w[i - 2].ror(10);
//         w[i] = w[i - 16] + s0 + w[i - 7] + s1;
//     }

//     waffle::TurboProver prover = composer.preprocess();
//     waffle::TurboVerifier verifier = composer.create_verifier();

//     waffle::plonk_proof proof = prover.construct_proof();

//     bool result = verifier.verify_proof(proof);
//     EXPECT_EQ(result, true);
// }

// TEST(extended_composer, logic_operations)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();

//     uint32 e = witness_t(&composer, 0xabcdefU);
//     uint32 g = 0xffffffff;
//     ((~e) & g) + 1;

//     waffle::TurboProver prover = composer.preprocess();

//     waffle::TurboVerifier verifier = composer.create_verifier();

//     waffle::plonk_proof proof = prover.construct_proof();

//     bool result = verifier.verify_proof(proof);
//     EXPECT_EQ(result, true);
// }