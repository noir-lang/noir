#include "index.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

namespace aztec3::circuits::recursion {

using namespace aztec3::utils::types;

class play_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }
};

TEST_F(play_tests, circuit_play_app_circuit)
{
    Composer composer = Composer(barretenberg::srs::get_crs_factory());
    play_app_circuit(composer, 1, 2);
}

TEST_F(play_tests, circuit_play_app_proof_gen)
{
    Composer app_composer = Composer(barretenberg::srs::get_crs_factory());
    play_app_circuit(app_composer, 100, 200);

    if (app_composer.failed()) {
        info("Play app circuit logic failed: ", app_composer.err());
    }

    auto app_prover = app_composer.create_prover();
    proof const app_proof = app_prover.construct_proof();
    info("app_proof: ", app_proof.proof_data);
}

TEST_F(play_tests, circuit_play_recursive_proof_gen)
{
    Composer app_composer = Composer(barretenberg::srs::get_crs_factory());
    play_app_circuit(app_composer, 1, 2);

    if (app_composer.failed()) {
        info("Play app circuit logic failed: ", app_composer.err());
    }

    auto app_prover = app_composer.create_prover();
    proof const app_proof = app_prover.construct_proof();
    info("app_proof: ", app_proof.proof_data);

    std::shared_ptr<plonk::verification_key> const app_vk = app_composer.compute_verification_key();

    Composer recursive_composer = Composer(barretenberg::srs::get_crs_factory());
    auto aggregation_output = play_recursive_circuit(recursive_composer, app_vk, app_proof);

    if (recursive_composer.failed()) {
        info("Play recursive circuit logic failed: ", recursive_composer.err());
    }
}

TEST_F(play_tests, circuit_play_recursive_2_proof_gen)
{
    Composer app_composer = Composer(barretenberg::srs::get_crs_factory());
    play_app_circuit(app_composer, 1, 2);

    if (app_composer.failed()) {
        info("Play app circuit logic failed: ", app_composer.err());
    }

    auto app_prover = app_composer.create_prover();
    proof const app_proof = app_prover.construct_proof();
    std::shared_ptr<plonk::verification_key> const app_vk = app_composer.compute_verification_key();

    //*******************************************************************************

    Composer dummy_circuit_composer = Composer(barretenberg::srs::get_crs_factory());
    dummy_circuit(dummy_circuit_composer, 1, 2);

    if (dummy_circuit_composer.failed()) {
        info("dummy_circuit logic failed: ", dummy_circuit_composer.err());
    }

    auto dummy_circuit_prover = dummy_circuit_composer.create_prover();
    proof const dummy_circuit_proof = dummy_circuit_prover.construct_proof();
    std::shared_ptr<plonk::verification_key> const dummy_circuit_vk = dummy_circuit_composer.compute_verification_key();

    //*******************************************************************************

    Composer recursion_1_composer = Composer(barretenberg::srs::get_crs_factory(), 0);
    auto recursion_1_output =
        play_recursive_circuit_2(recursion_1_composer, app_vk, app_proof, dummy_circuit_vk, dummy_circuit_proof);

    if (recursion_1_composer.failed()) {
        info("recursion_1 circuit logic failed: ", recursion_1_composer.err());
    }

    auto recursion_1_prover = recursion_1_composer.create_prover();

    proof const recursion_1_proof = recursion_1_prover.construct_proof();

    std::shared_ptr<plonk::verification_key> const recursion_1_vk = recursion_1_composer.compute_verification_key();

    //*******************************************************************************

    // Composer recursion_2_composer = Composer(barretenberg::srs::get_crs_factory());
    // aggregation_output<bn254> recursion_2_output = play_recursive_circuit_2<TurboRecursion>(
    //     recursion_2_composer, app_vk, app_proof, recursion_1_vk, recursion_1_proof);

    // if (recursion_2_composer.failed()) {
    //     info("recursion_2 circuit logic failed: ", recursion_2_composer.err());
    // }

    // Prover recursion_2_prover = recursion_2_composer.create_prover();
    // proof recursion_2_proof = recursion_2_prover.construct_proof();
    // std::shared_ptr<plonk::verification_key> recursion_2_vk =
    // recursion_2_composer.compute_verification_key();
}

}  // namespace aztec3::circuits::recursion
