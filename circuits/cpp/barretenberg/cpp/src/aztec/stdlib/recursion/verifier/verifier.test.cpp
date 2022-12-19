#include "verifier.hpp"
#include <common/test.hpp>
#include <plonk/transcript/transcript.hpp>
#include <proof_system/proving_key/serialize.hpp>
#include <stdlib/primitives/curves/bn254.hpp>
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include "../../hash/blake3s/blake3s.hpp"
#include "../../hash/pedersen/pedersen.hpp"
#include "program_settings.hpp"

using namespace plonk;

template <typename OuterComposer> class stdlib_verifier : public testing::Test {
    using InnerComposer = waffle::UltraComposer;

    typedef stdlib::bn254<InnerComposer> inner_curve;
    typedef stdlib::bn254<OuterComposer> outer_curve;
    typedef plonk::stdlib::recursion::verification_key<outer_curve> verification_key_pt;
    typedef plonk::stdlib::recursion::recursive_ultra_verifier_settings<outer_curve> recursive_settings;
    typedef plonk::stdlib::recursion::recursive_ultra_to_standard_verifier_settings<outer_curve>
        ultra_to_standard_recursive_settings;
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
        plonk::stdlib::pedersen<InnerComposer>::compress(a, b);
        typename inner_curve::byte_array_ct to_hash(&composer, "nonsense test data");
        stdlib::blake3s(to_hash);

        barretenberg::fr bigfield_data = fr::random_element();
        barretenberg::fr bigfield_data_a{ bigfield_data.data[0], bigfield_data.data[1], 0, 0 };
        barretenberg::fr bigfield_data_b{ bigfield_data.data[2], bigfield_data.data[3], 0, 0 };

        typename inner_curve::fq_ct big_a(fr_ct(witness_ct(&composer, bigfield_data_a.to_montgomery_form())),
                                          fr_ct(witness_ct(&composer, 0)));
        typename inner_curve::fq_ct big_b(fr_ct(witness_ct(&composer, bigfield_data_b.to_montgomery_form())),
                                          fr_ct(witness_ct(&composer, 0)));

        big_a* big_b;
    };

    /**
     * Test is included because UltraComposer used to fail for circuits which didn't lookup any tables.
     */
    static void create_inner_circuit_no_tables(InnerComposer& composer,
                                               const std::vector<barretenberg::fr>& public_inputs)
    {
        // A nice Pythagorean triples circuit example: "I know a & b s.t. a^2 + b^2 = c^2".
        fr_ct a(witness_ct(&composer, public_inputs[0]));
        fr_ct b(witness_ct(&composer, public_inputs[1]));
        fr_ct c(witness_ct(&composer, public_inputs[2]));

        auto a_sq = a * a;
        auto b_sq = b * b;
        auto c_sq = c * c;

        (c_sq).assert_equal(a_sq + b_sq);

        c_sq.set_public();
    };

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
        stdlib::blake3s(to_hash);

        barretenberg::fr bigfield_data = fr::random_element();
        barretenberg::fr bigfield_data_a{ bigfield_data.data[0], bigfield_data.data[1], 0, 0 };
        barretenberg::fr bigfield_data_b{ bigfield_data.data[2], bigfield_data.data[3], 0, 0 };

        inner_curve::bn254::fq_ct big_a(fr_ct(witness_ct(&composer, bigfield_data_a.to_montgomery_form())),
                                        fr_ct(witness_ct(&composer, 0)));
        inner_curve::bn254::fq_ct big_b(fr_ct(witness_ct(&composer, bigfield_data_b.to_montgomery_form())),
                                        fr_ct(witness_ct(&composer, 0)));
        ((big_a * big_b) + big_a) * big_b;
    }

    static circuit_outputs create_outer_circuit(InnerComposer& inner_composer, OuterComposer& outer_composer)
    {
        // These constexpr definitions are to allow for the following:
        // An Ultra Pedersen hash evaluates to a different value from the Turbo/Standard versions of the Pedersen hash.
        // Therefore, the fiat-shamir challenges generated by the prover and verifier _could_ accidentally be different
        // if an ultra proof is generated using ultra-pedersen challenges, but is being verified within a non-ultra
        // circuit which uses non-ultra-pedersen challenges. We need the prover and verifier hashes to be the same. The
        // solution is to select the relevant prover and verifier types (whose settings use the same hash for
        // fiat-shamir), depending on the Inner-Outer combo. It's a bit clunky, but the alternative is to have a
        // template argument for the hashtype, and that would pervade the entire UltraComposer, which would be
        // horrendous.
        constexpr bool is_ultra_to_ultra = std::is_same<OuterComposer, waffle::UltraComposer>::value;
        typedef typename std::conditional<is_ultra_to_ultra,
                                          waffle::UnrolledUltraProver,
                                          waffle::UnrolledUltraToStandardProver>::type ProverOfInnerCircuit;
        typedef typename std::conditional<is_ultra_to_ultra,
                                          waffle::UnrolledUltraVerifier,
                                          waffle::UnrolledUltraToStandardVerifier>::type VerifierOfInnerProof;
        typedef
            typename std::conditional<is_ultra_to_ultra, recursive_settings, ultra_to_standard_recursive_settings>::type
                RecursiveSettings;

        info("Creating ultra (inner) unrolled prover...");
        ProverOfInnerCircuit prover;
        if constexpr (is_ultra_to_ultra) {
            prover = inner_composer.create_unrolled_prover();
        } else {
            prover = inner_composer.create_unrolled_ultra_to_standard_prover();
        }

        info("Computing verification key...");
        const auto verification_key_native = inner_composer.compute_verification_key();
        // Convert the verification key's elements into _circuit_ types, using the OUTER composer.
        std::shared_ptr<verification_key_pt> verification_key =
            verification_key_pt::from_witness(&outer_composer, verification_key_native);

        info("Constructing the ultra (inner) proof ...");
        waffle::plonk_proof recursive_proof = prover.construct_proof();

        {
            // Native check is mainly for comparison vs circuit version of the verifier.
            info("Creating a native ultra (inner) verifier...");
            VerifierOfInnerProof native_verifier;

            if constexpr (is_ultra_to_ultra) {
                native_verifier = inner_composer.create_unrolled_verifier();
            } else {
                native_verifier = inner_composer.create_unrolled_ultra_to_standard_verifier();
            }

            info("Verifying the ultra (inner) proof natively...");
            auto native_result = native_verifier.verify_proof(recursive_proof);

            info("Native result: ", native_result);
        }

        transcript::Manifest recursive_manifest =
            InnerComposer::create_unrolled_manifest(prover.key->num_public_inputs);

        info("Verifying the ultra (inner) proof with CIRCUIT TYPES (i.e. within a standard plonk arithmetic circuit):");
        stdlib::recursion::recursion_output<outer_curve> output =
            stdlib::recursion::verify_proof<outer_curve, RecursiveSettings>(
                &outer_composer, verification_key, recursive_manifest, recursive_proof);

        return { output, verification_key };
    };

    static circuit_outputs create_double_outer_circuit(InnerComposer& inner_composer_a,
                                                       InnerComposer& inner_composer_b,
                                                       OuterComposer& outer_composer)
    {
        // See create_outer_circuit for explanation of these constexpr definitions.
        constexpr bool is_ultra_to_ultra = std::is_same<OuterComposer, waffle::UltraComposer>::value;
        typedef typename std::conditional<is_ultra_to_ultra,
                                          waffle::UnrolledUltraProver,
                                          waffle::UnrolledUltraToStandardProver>::type ProverOfInnerCircuit;
        typedef
            typename std::conditional<is_ultra_to_ultra, recursive_settings, ultra_to_standard_recursive_settings>::type
                RecursiveSettings;

        ProverOfInnerCircuit prover;
        if constexpr (is_ultra_to_ultra) {
            prover = inner_composer_a.create_unrolled_prover();
        } else {
            prover = inner_composer_a.create_unrolled_ultra_to_standard_prover();
        }

        const auto verification_key_native = inner_composer_a.compute_verification_key();
        std::shared_ptr<verification_key_pt> verification_key =
            verification_key_pt::from_witness(&outer_composer, verification_key_native);

        waffle::plonk_proof recursive_proof_a = prover.construct_proof();

        transcript::Manifest recursive_manifest =
            InnerComposer::create_unrolled_manifest(prover.key->num_public_inputs);

        stdlib::recursion::recursion_output<outer_curve> previous_output =
            stdlib::recursion::verify_proof<outer_curve, RecursiveSettings>(
                &outer_composer, verification_key, recursive_manifest, recursive_proof_a);

        if constexpr (is_ultra_to_ultra) {
            prover = inner_composer_b.create_unrolled_prover();
        } else {
            prover = inner_composer_b.create_unrolled_ultra_to_standard_prover();
        }

        const auto verification_key_b_raw = inner_composer_b.compute_verification_key();
        std::shared_ptr<verification_key_pt> verification_key_b =
            verification_key_pt::from_witness(&outer_composer, verification_key_b_raw);

        waffle::plonk_proof recursive_proof_b = prover.construct_proof();

        stdlib::recursion::recursion_output<outer_curve> output =
            stdlib::recursion::verify_proof<outer_curve, RecursiveSettings>(
                &outer_composer, verification_key_b, recursive_manifest, recursive_proof_b, previous_output);

        return { output, verification_key };
    }

    // creates a cicuit that verifies either a proof from composer a, or from composer b
    static circuit_outputs create_outer_circuit_with_variable_inner_circuit(InnerComposer& inner_composer_a,
                                                                            InnerComposer& inner_composer_b,
                                                                            OuterComposer& outer_composer,
                                                                            const bool proof_type,
                                                                            const bool create_failing_proof = false,
                                                                            const bool use_constant_key = false)
    {
        // See create_outer_circuit for explanation of these constexpr definitions.
        constexpr bool is_ultra_to_ultra = std::is_same<OuterComposer, waffle::UltraComposer>::value;
        typedef typename std::conditional<is_ultra_to_ultra,
                                          waffle::UnrolledUltraProver,
                                          waffle::UnrolledUltraToStandardProver>::type ProverOfInnerCircuit;
        typedef
            typename std::conditional<is_ultra_to_ultra, recursive_settings, ultra_to_standard_recursive_settings>::type
                RecursiveSettings;

        ProverOfInnerCircuit prover_a;
        ProverOfInnerCircuit prover_b;
        if constexpr (is_ultra_to_ultra) {
            prover_a = inner_composer_a.create_unrolled_prover();
            prover_b = inner_composer_b.create_unrolled_prover();
        } else {
            prover_a = inner_composer_a.create_unrolled_ultra_to_standard_prover();
            prover_b = inner_composer_b.create_unrolled_ultra_to_standard_prover();
        }

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
            stdlib::recursion::verify_proof<outer_curve, RecursiveSettings>(
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

        EXPECT_EQ(outer_composer.failed(), false);

        info("creating prover for outer circuit");
        info("composer gates = ", outer_composer.get_num_gates());
        auto prover = outer_composer.create_prover();
        info("created prover for outer circuit");

        info("creating verifier for outer proof");
        auto verifier = outer_composer.create_verifier();

        info("creating outer proof for outer circuit");
        waffle::plonk_proof proof = prover.construct_proof();
        info("created outer proof");

        info("verifying the outer proof");
        bool result = verifier.verify_proof(proof);
        info("Outer proof verification result: ", result);

        EXPECT_EQ(result, true);
    }

    static void test_recursive_proof_composition_ultra_no_tables()
    {
        InnerComposer inner_composer = InnerComposer("../srs_db/ignition");
        OuterComposer outer_composer = OuterComposer("../srs_db/ignition");

        std::vector<barretenberg::fr> inner_inputs{ 3, 4, 5 };

        create_inner_circuit_no_tables(inner_composer, inner_inputs);

        auto circuit_output = create_outer_circuit(inner_composer, outer_composer);

        g1::affine_element P[2];
        P[0].x = barretenberg::fq(circuit_output.recursion_output.P0.x.get_value().lo);
        P[0].y = barretenberg::fq(circuit_output.recursion_output.P0.y.get_value().lo);
        P[1].x = barretenberg::fq(circuit_output.recursion_output.P1.x.get_value().lo);
        P[1].y = barretenberg::fq(circuit_output.recursion_output.P1.y.get_value().lo);

        barretenberg::fq12 inner_proof_result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
            P, circuit_output.verification_key->reference_string->get_precomputed_g2_lines(), 2);

        EXPECT_EQ(inner_proof_result, barretenberg::fq12::one());

        circuit_output.recursion_output.add_proof_outputs_as_public_inputs();

        EXPECT_EQ(outer_composer.failed(), false);

        info("creating prover for outer circuit");
        info("composer gates = ", outer_composer.get_num_gates());
        auto prover = outer_composer.create_prover();
        info("created prover for outer circuit");

        info("creating verifier for outer proof");
        auto verifier = outer_composer.create_verifier();

        info("creating outer proof for outer circuit");
        waffle::plonk_proof proof = prover.construct_proof();
        info("created outer proof");

        info("verifying the outer proof");
        bool result = verifier.verify_proof(proof);
        info("Outer proof verification result: ", result);

        EXPECT_EQ(result, true);
    }

    static void test_double_verification()
    {
        if constexpr (std::is_same<OuterComposer, waffle::StandardComposer>::value)
            return; // We only care about running this test for turbo and ultra outer circuits, since in practice the
                    // only circuits which verify >1 proof are ultra or turbo circuits. Standard uses so many gates
                    // (16m) that it's a waste of time testing it.

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

    static void test_inner_circuit()
    {
        if constexpr (!std::is_same<OuterComposer, waffle::StandardComposer>::value)
            return; // We only want to run this test once (since it's not actually dependent on the typed test
                    // parameter; which is the outer composer). We've only made it a typed test so that it can be
                    // included in this test suite. So to avoid running this test identically 3 times, we escape all but
                    // 1 permutation.

        InnerComposer inner_composer = InnerComposer("../srs_db/ignition");
        std::vector<barretenberg::fr> inner_inputs{ barretenberg::fr::random_element(),
                                                    barretenberg::fr::random_element(),
                                                    barretenberg::fr::random_element() };

        create_inner_circuit(inner_composer, inner_inputs);

        auto prover = inner_composer.create_unrolled_prover();
        auto verifier = inner_composer.create_unrolled_verifier();
        auto proof = prover.construct_proof();
        auto verified = verifier.verify_proof(proof);
        EXPECT_EQ(verified, true);
    }
};

typedef testing::Types<waffle::StandardComposer, waffle::TurboComposer, waffle::UltraComposer> OuterComposerTypes;

TYPED_TEST_SUITE(stdlib_verifier, OuterComposerTypes);

HEAVY_TYPED_TEST(stdlib_verifier, recursive_proof_composition)
{
    TestFixture::test_recursive_proof_composition();
};

HEAVY_TYPED_TEST(stdlib_verifier, recursive_proof_composition_ultra_no_tables)
{
    TestFixture::test_recursive_proof_composition_ultra_no_tables();
};

// CircleCI can't cope with this.
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

HEAVY_TYPED_TEST(stdlib_verifier, test_inner_circuit)
{
    TestFixture::test_inner_circuit();
}
