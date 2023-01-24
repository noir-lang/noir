#include "verifier.hpp"
#include <common/test.hpp>
#include <plonk/transcript/transcript.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <stdlib/primitives/curves/bn254.hpp>
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include "../../hash/blake2s/blake2s.hpp"
#include "../../hash/pedersen/pedersen.hpp"
#include "program_settings.hpp"

using namespace plonk;

template <typename OuterComposer> class stdlib_verifier : public testing::Test {
    using InnerComposer = waffle::TurboComposer;

    typedef stdlib::bn254<InnerComposer> inner_curve;
    typedef stdlib::bn254<OuterComposer> outer_curve;
    typedef plonk::stdlib::recursion::verification_key<outer_curve> verification_key_pt;
    typedef plonk::stdlib::recursion::recursive_turbo_verifier_settings<outer_curve> recursive_settings;
    typedef inner_curve::fr_ct fr_ct;
    typedef inner_curve::public_witness_ct public_witness_ct;
    typedef inner_curve::witness_ct witness_ct;

    struct circuit_outputs {
        stdlib::recursion::recursion_output<outer_curve> recursion_output;
        std::shared_ptr<verification_key_pt> verification_key;
    };

    static void create_inner_circuit(InnerComposer& composer, const std::vector<barretenberg::fr>& public_inputs)
    {
        fr_ct a(public_witness_ct(&composer, public_inputs[0]));
        fr_ct b(public_witness_ct(&composer, public_inputs[1]));
        fr_ct c(public_witness_ct(&composer, public_inputs[2]));

        for (size_t i = 0; i < 32; ++i) {
            a = (a * b) + b + a;
            a = a.madd(b, c);
        }
        plonk::stdlib::pedersen<waffle::TurboComposer>::compress(a, b);
        typename inner_curve::byte_array_ct to_hash(&composer, "nonsense test data");
        stdlib::blake2s(to_hash);

        barretenberg::fr bigfield_data = fr::random_element();
        barretenberg::fr bigfield_data_a{ bigfield_data.data[0], bigfield_data.data[1], 0, 0 };
        barretenberg::fr bigfield_data_b{ bigfield_data.data[2], bigfield_data.data[3], 0, 0 };

        typename inner_curve::fq_ct big_a(fr_ct(witness_ct(&composer, bigfield_data_a.to_montgomery_form())),
                                          fr_ct(witness_ct(&composer, 0)));
        typename inner_curve::fq_ct big_b(fr_ct(witness_ct(&composer, bigfield_data_b.to_montgomery_form())),
                                          fr_ct(witness_ct(&composer, 0)));
        big_a* big_b;
    };

    static circuit_outputs create_outer_circuit(InnerComposer& inner_composer, OuterComposer& outer_composer)
    {
        auto prover = inner_composer.create_unrolled_prover();
        const auto verification_key_raw = inner_composer.compute_verification_key();
        std::shared_ptr<verification_key_pt> verification_key =
            verification_key_pt::from_witness(&outer_composer, verification_key_raw);
        waffle::plonk_proof recursive_proof = prover.construct_proof();
        transcript::Manifest recursive_manifest =
            InnerComposer::create_unrolled_manifest(prover.key->num_public_inputs);
        stdlib::recursion::recursion_output<outer_curve> output =
            stdlib::recursion::verify_proof<outer_curve, recursive_settings>(
                &outer_composer, verification_key, recursive_manifest, recursive_proof);
        return { output, verification_key };
    };

    static circuit_outputs create_double_outer_circuit(InnerComposer& inner_composer_a,
                                                       InnerComposer& inner_composer_b,
                                                       OuterComposer& outer_composer)
    {

        auto prover = inner_composer_a.create_unrolled_prover();

        const auto verification_key_raw = inner_composer_a.compute_verification_key();
        std::shared_ptr<verification_key_pt> verification_key =
            verification_key_pt::from_witness(&outer_composer, verification_key_raw);
        waffle::plonk_proof recursive_proof_a = prover.construct_proof();

        transcript::Manifest recursive_manifest =
            InnerComposer::create_unrolled_manifest(prover.key->num_public_inputs);

        stdlib::recursion::recursion_output<outer_curve> previous_output =
            stdlib::recursion::verify_proof<outer_curve, recursive_settings>(
                &outer_composer, verification_key, recursive_manifest, recursive_proof_a);

        auto prover_b = inner_composer_b.create_unrolled_prover();

        const auto verification_key_b_raw = inner_composer_b.compute_verification_key();
        std::shared_ptr<verification_key_pt> verification_key_b =
            verification_key_pt::from_witness(&outer_composer, verification_key_b_raw);
        waffle::plonk_proof recursive_proof_b = prover_b.construct_proof();

        stdlib::recursion::recursion_output<outer_curve> output =
            stdlib::recursion::verify_proof<outer_curve, recursive_settings>(
                &outer_composer, verification_key_b, recursive_manifest, recursive_proof_b, previous_output);

        return { output, verification_key };
    }

    static void create_alternate_inner_circuit(InnerComposer& composer,
                                               const std::vector<barretenberg::fr>& public_inputs)
    {
        fr_ct a(public_witness_ct(&composer, public_inputs[0]));
        fr_ct b(public_witness_ct(&composer, public_inputs[1]));
        fr_ct c(public_witness_ct(&composer, public_inputs[2]));

        for (size_t i = 0; i < 32; ++i) {
            a = (a * b) + b + a;
            a = c.madd(b, a);
        }
        plonk::stdlib::pedersen<InnerComposer>::compress(a, a);
        inner_curve::byte_array_ct to_hash(&composer, "different nonsense test data");
        stdlib::blake2s(to_hash);

        barretenberg::fr bigfield_data = fr::random_element();
        barretenberg::fr bigfield_data_a{ bigfield_data.data[0], bigfield_data.data[1], 0, 0 };
        barretenberg::fr bigfield_data_b{ bigfield_data.data[2], bigfield_data.data[3], 0, 0 };

        inner_curve::bn254::fq_ct big_a(fr_ct(witness_ct(&composer, bigfield_data_a.to_montgomery_form())),
                                        fr_ct(witness_ct(&composer, 0)));
        inner_curve::bn254::fq_ct big_b(fr_ct(witness_ct(&composer, bigfield_data_b.to_montgomery_form())),
                                        fr_ct(witness_ct(&composer, 0)));
        ((big_a * big_b) + big_a) * big_b;
    }

    // creates a cicuit that verifies either a proof from composer a, or from composer b
    static circuit_outputs create_outer_circuit_with_variable_inner_circuit(InnerComposer& inner_composer_a,
                                                                            InnerComposer& inner_composer_b,
                                                                            OuterComposer& outer_composer,
                                                                            const bool proof_type,
                                                                            const bool create_failing_proof = false,
                                                                            const bool use_constant_key = false)
    {
        auto prover_a = inner_composer_a.create_unrolled_prover();
        auto prover_b = inner_composer_b.create_unrolled_prover();
        const auto verification_key_raw_a = inner_composer_a.compute_verification_key();
        const auto verification_key_raw_b = inner_composer_b.compute_verification_key();

        std::shared_ptr<verification_key_pt> verification_key;
        if (use_constant_key) {
            verification_key = proof_type
                                   ? verification_key_pt::from_constants(&outer_composer, verification_key_raw_a)
                                   : verification_key_pt::from_constants(&outer_composer, verification_key_raw_b);

        } else {
            verification_key = proof_type ? verification_key_pt::from_witness(&outer_composer, verification_key_raw_a)
                                          : verification_key_pt::from_witness(&outer_composer, verification_key_raw_b);
        }
        if (!use_constant_key) {
            if (create_failing_proof) {
                verification_key->validate_key_is_in_set({ verification_key_raw_b, verification_key_raw_b });
            } else {
                verification_key->validate_key_is_in_set({ verification_key_raw_a, verification_key_raw_b });
            }
        }
        waffle::plonk_proof recursive_proof = proof_type ? prover_a.construct_proof() : prover_b.construct_proof();
        transcript::Manifest recursive_manifest =
            InnerComposer::create_unrolled_manifest(prover_a.key->num_public_inputs);
        stdlib::recursion::recursion_output<outer_curve> output =
            stdlib::recursion::verify_proof<outer_curve, recursive_settings>(
                &outer_composer, verification_key, recursive_manifest, recursive_proof);
        return { output, verification_key };
    }

  public:
    static void test_recursive_proof_composition()
    {
        InnerComposer inner_composer = InnerComposer("../srs_db/ignition");
        OuterComposer outer_composer = OuterComposer("../srs_db/ignition");
        std::vector<barretenberg::fr> inner_inputs{ barretenberg::fr::random_element(),
                                                    barretenberg::fr::random_element(),
                                                    barretenberg::fr::random_element() };

        create_inner_circuit(inner_composer, inner_inputs);

        auto circuit_output = create_outer_circuit(inner_composer, outer_composer);

        g1::affine_element P[2];
        P[0].x = barretenberg::fq(circuit_output.recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(circuit_output.recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(circuit_output.recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(circuit_output.recursion_output.P1.y.get_value().lo);
        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, circuit_output.verification_key->reference_string->get_precomputed_g2_lines(), 2);

        EXPECT_EQ(circuit_output.recursion_output.public_inputs[0].get_value(), inner_inputs[0]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[1].get_value(), inner_inputs[1]);

        EXPECT_EQ(inner_proof_result, barretenberg::fq12::one());

        circuit_output.recursion_output.add_proof_outputs_as_public_inputs();

        EXPECT_EQ(outer_composer.failed, false);
        std::cout << "creating prover" << std::endl;
        std::cout << "composer gates = " << outer_composer.get_num_gates() << std::endl;
        auto prover = outer_composer.create_prover();
        std::cout << "created prover" << std::endl;

        std::cout << "creating verifier" << std::endl;
        auto verifier = outer_composer.create_verifier();

        std::cout << "validated. creating proof" << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        std::cout << "created proof" << std::endl;

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_double_verification()
    {
        InnerComposer inner_composer_a = InnerComposer("../srs_db/ignition");
        InnerComposer inner_composer_b = InnerComposer("../srs_db/ignition");

        OuterComposer outer_composer = OuterComposer("../srs_db/ignition");

        std::vector<barretenberg::fr> inner_inputs{ barretenberg::fr::random_element(),
                                                    barretenberg::fr::random_element(),
                                                    barretenberg::fr::random_element() };

        create_inner_circuit(inner_composer_a, inner_inputs);
        create_inner_circuit(inner_composer_b, inner_inputs);

        auto circuit_output = create_double_outer_circuit(inner_composer_a, inner_composer_b, outer_composer);

        g1::affine_element P[2];
        P[0].x = barretenberg::fq(circuit_output.recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(circuit_output.recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(circuit_output.recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(circuit_output.recursion_output.P1.y.get_value().lo);
        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, circuit_output.verification_key->reference_string->get_precomputed_g2_lines(), 2);

        EXPECT_EQ(circuit_output.recursion_output.public_inputs[0].get_value(), inner_inputs[0]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[1].get_value(), inner_inputs[1]);

        EXPECT_EQ(inner_proof_result, barretenberg::fq12::one());

        printf("composer gates = %zu\n", outer_composer.get_num_gates());

        std::cout << "creating prover" << std::endl;
        auto prover = outer_composer.create_prover();
        std::cout << "created prover" << std::endl;

        std::cout << "creating verifier" << std::endl;
        auto verifier = outer_composer.create_verifier();

        std::cout << "validated. creating proof" << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        std::cout << "created proof" << std::endl;

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    // verifies a proof of a circuit that verifies one of two proofs. Test 'a' uses a proof over the first of the two
    // variable circuits
    static void test_recursive_proof_composition_with_variable_verification_key_a()
    {
        InnerComposer inner_composer_a = InnerComposer("../srs_db/ignition");
        InnerComposer inner_composer_b = InnerComposer("../srs_db/ignition");
        OuterComposer outer_composer = OuterComposer("../srs_db/ignition");
        std::vector<barretenberg::fr> inner_inputs_a{ barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element() };

        std::vector<barretenberg::fr> inner_inputs_b{ barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element() };

        create_inner_circuit(inner_composer_a, inner_inputs_a);
        create_alternate_inner_circuit(inner_composer_b, inner_inputs_b);

        auto circuit_output =
            create_outer_circuit_with_variable_inner_circuit(inner_composer_a, inner_composer_b, outer_composer, true);

        g1::affine_element P[2];
        P[0].x = barretenberg::fq(circuit_output.recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(circuit_output.recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(circuit_output.recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(circuit_output.recursion_output.P1.y.get_value().lo);
        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, circuit_output.verification_key->reference_string->get_precomputed_g2_lines(), 2);

        EXPECT_EQ(circuit_output.recursion_output.public_inputs[0].get_value(), inner_inputs_a[0]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[1].get_value(), inner_inputs_a[1]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[2].get_value(), inner_inputs_a[2]);

        EXPECT_EQ(inner_proof_result, barretenberg::fq12::one());

        printf("composer gates = %zu\n", outer_composer.get_num_gates());

        auto prover = outer_composer.create_prover();

        auto verifier = outer_composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    // verifies a proof of a circuit that verifies one of two proofs. Test 'b' uses a proof over the second of the two
    // variable circuits
    static void test_recursive_proof_composition_with_variable_verification_key_b()
    {
        InnerComposer inner_composer_a = InnerComposer("../srs_db/ignition");
        InnerComposer inner_composer_b = InnerComposer("../srs_db/ignition");
        OuterComposer outer_composer = OuterComposer("../srs_db/ignition");
        std::vector<barretenberg::fr> inner_inputs_a{ barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element() };

        std::vector<barretenberg::fr> inner_inputs_b{ barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element() };

        create_inner_circuit(inner_composer_a, inner_inputs_a);
        create_alternate_inner_circuit(inner_composer_b, inner_inputs_b);
        auto circuit_output =
            create_outer_circuit_with_variable_inner_circuit(inner_composer_a, inner_composer_b, outer_composer, false);
        g1::affine_element P[2];

        P[0].x = barretenberg::fq(circuit_output.recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(circuit_output.recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(circuit_output.recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(circuit_output.recursion_output.P1.y.get_value().lo);

        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, circuit_output.verification_key->reference_string->get_precomputed_g2_lines(), 2);

        EXPECT_EQ(circuit_output.recursion_output.public_inputs[0].get_value(), inner_inputs_b[0]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[1].get_value(), inner_inputs_b[1]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[2].get_value(), inner_inputs_b[2]);

        EXPECT_EQ(inner_proof_result, barretenberg::fq12::one());

        printf("composer gates = %zu\n", outer_composer.get_num_gates());

        auto prover = outer_composer.create_prover();

        auto verifier = outer_composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_recursive_proof_composition_with_variable_verification_key_failure_case()
    {
        InnerComposer inner_composer_a = InnerComposer("../srs_db/ignition");
        InnerComposer inner_composer_b = InnerComposer("../srs_db/ignition");
        OuterComposer outer_composer = OuterComposer("../srs_db/ignition");
        std::vector<barretenberg::fr> inner_inputs_a{ barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element() };

        std::vector<barretenberg::fr> inner_inputs_b{ barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element() };

        create_inner_circuit(inner_composer_a, inner_inputs_a);
        create_alternate_inner_circuit(inner_composer_b, inner_inputs_b);

        auto circuit_output = create_outer_circuit_with_variable_inner_circuit(
            inner_composer_a, inner_composer_b, outer_composer, true, true);

        g1::affine_element P[2];
        P[0].x = barretenberg::fq(circuit_output.recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(circuit_output.recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(circuit_output.recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(circuit_output.recursion_output.P1.y.get_value().lo);
        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, circuit_output.verification_key->reference_string->get_precomputed_g2_lines(), 2);

        EXPECT_EQ(circuit_output.recursion_output.public_inputs[0].get_value(), inner_inputs_a[0]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[1].get_value(), inner_inputs_a[1]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[2].get_value(), inner_inputs_a[2]);

        EXPECT_EQ(inner_proof_result, barretenberg::fq12::one());

        printf("composer gates = %zu\n", outer_composer.get_num_gates());

        auto prover = outer_composer.create_prover();

        auto verifier = outer_composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }

    static void test_recursive_proof_composition_with_constant_verification_key()
    {
        InnerComposer inner_composer_a = InnerComposer("../srs_db/ignition");
        InnerComposer inner_composer_b = InnerComposer("../srs_db/ignition");
        OuterComposer outer_composer = OuterComposer("../srs_db/ignition");
        std::vector<barretenberg::fr> inner_inputs_a{ barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element() };

        std::vector<barretenberg::fr> inner_inputs_b{ barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element(),
                                                      barretenberg::fr::random_element() };

        create_inner_circuit(inner_composer_a, inner_inputs_a);
        create_alternate_inner_circuit(inner_composer_b, inner_inputs_b);

        auto circuit_output = create_outer_circuit_with_variable_inner_circuit(
            inner_composer_a, inner_composer_b, outer_composer, true, false, true);

        g1::affine_element P[2];
        P[0].x = barretenberg::fq(circuit_output.recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(circuit_output.recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(circuit_output.recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(circuit_output.recursion_output.P1.y.get_value().lo);
        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, circuit_output.verification_key->reference_string->get_precomputed_g2_lines(), 2);

        EXPECT_EQ(circuit_output.recursion_output.public_inputs[0].get_value(), inner_inputs_a[0]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[1].get_value(), inner_inputs_a[1]);
        EXPECT_EQ(circuit_output.recursion_output.public_inputs[2].get_value(), inner_inputs_a[2]);

        EXPECT_EQ(inner_proof_result, barretenberg::fq12::one());

        printf("composer gates = %zu\n", outer_composer.get_num_gates());

        auto prover = outer_composer.create_prover();

        auto verifier = outer_composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
};

typedef testing::Types<waffle::StandardComposer, waffle::TurboComposer> OuterComposerTypes;

TYPED_TEST_SUITE(stdlib_verifier, OuterComposerTypes);

HEAVY_TYPED_TEST(stdlib_verifier, recursive_proof_composition)
{
    TestFixture::test_recursive_proof_composition();
};

// Produces a huge 16m gate circuit with the StandardComposer. Really needed?
// HEAVY_TYPED_TEST(stdlib_verifier, double_verification)
// {
//     TestFixture::test_double_verification();
// };

HEAVY_TYPED_TEST(stdlib_verifier, recursive_proof_composition_with_variable_verification_key_a)
{
    TestFixture::test_recursive_proof_composition_with_variable_verification_key_a();
}

HEAVY_TYPED_TEST(stdlib_verifier, recursive_proof_composition_with_variable_verification_key_b)
{
    TestFixture::test_recursive_proof_composition_with_variable_verification_key_b();
}

HEAVY_TYPED_TEST(stdlib_verifier, recursive_proof_composition_var_verif_key_fail)
{
    TestFixture::test_recursive_proof_composition_with_variable_verification_key_failure_case();
}

HEAVY_TYPED_TEST(stdlib_verifier, recursive_proof_composition_const_verif_key)
{
    TestFixture::test_recursive_proof_composition_with_constant_verification_key();
}