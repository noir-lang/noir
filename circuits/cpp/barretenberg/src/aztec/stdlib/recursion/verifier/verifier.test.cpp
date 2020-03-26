#include "verifier.hpp"
#include <gtest/gtest.h>

#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/bn254/g1.hpp>

#include <plonk/transcript/transcript.hpp>
#include <stdlib/types/turbo.hpp>

#include "program_settings.hpp"

using namespace plonk;

using namespace plonk::stdlib::types::turbo;

void create_inner_circuit(waffle::TurboComposer& composer)
{
    field_ct a(witness_ct(&composer, barretenberg::fr::random_element()));
    field_ct b(witness_ct(&composer, barretenberg::fr::random_element()));
    for (size_t i = 0; i < 32; ++i) {
        a = (a * b) + b + a;
        a = a.madd(b, a);
    }
}

// Ok, so we need to create a recursive circuit...
stdlib::recursion::recursion_output<group_ct> create_outer_circuit(waffle::TurboComposer& inner_composer,
                                                                   waffle::TurboComposer& outer_composer)
{
    // (ノಠ益ಠ)ノ彡┻━┻
    waffle::UnrolledTurboProver prover = inner_composer.create_unrolled_prover();

    std::shared_ptr<waffle::verification_key> verification_key = inner_composer.compute_verification_key();

    waffle::plonk_proof recursive_proof = prover.construct_proof();

    transcript::Manifest recursive_manifest =
        waffle::TurboComposer::create_unrolled_manifest(prover.key->num_public_inputs);

    stdlib::recursion::recursion_output<group_ct> output =
        stdlib::recursion::verify_proof<waffle::TurboComposer,
                                        plonk::stdlib::recursion::recursive_turbo_verifier_settings>(
            &outer_composer, verification_key, recursive_manifest, recursive_proof);
    return output;
}

TEST(stdlib_verifier, test_recursive_proof_composition)
{
    waffle::TurboComposer inner_composer = waffle::TurboComposer();
    waffle::TurboComposer outer_composer = waffle::TurboComposer();
    create_inner_circuit(inner_composer);
    create_outer_circuit(inner_composer, outer_composer);

    printf("composer gates = %zu\n", outer_composer.get_num_gates());
    waffle::TurboProver prover = outer_composer.create_prover();

    waffle::TurboVerifier verifier = outer_composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}