#include "index.hpp"
#include <common/test.hpp>
// #include <numeric/random/engine.hpp>
#include <stdlib/types/turbo.hpp>

namespace aztec3::circuits::recursion {

using namespace plonk::stdlib::types::turbo;
using plonk::stdlib::recursion::recursion_output;

// namespace {
// std::shared_ptr<waffle::DynamicFileReferenceStringFactory> srs;
// private_kernel::circuit_data private_kernel_cd;
// private_circuit::circuit_data private_circuit_cd;
// } // namespace

class play_tests : public ::testing::Test {
    //   protected:
    //     static void SetUpTestCase()
    //     {
    //         std::string CRS_PATH = "../srs_db/ignition";
    //         srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
    //         private_circuit_cd = join_split::get_circuit_data(srs);
    //         private_kernel_cd = claim::get_circuit_data(srs);
    //     }
};

TEST(play_tests, test_play_app_circuit)
{
    Composer composer;
    play_app_circuit(composer, 1, 2);
}

TEST(play_tests, test_play_app_proof_gen)
{
    Composer app_composer;
    play_app_circuit(app_composer, 100, 200);

    if (app_composer.failed) {
        info("Play app circuit logic failed: ", app_composer.err);
    }

    UnrolledProver app_prover = app_composer.create_unrolled_prover();
    waffle::plonk_proof app_proof = app_prover.construct_proof();
    info("app_proof: ", app_proof.proof_data);
}

TEST(play_tests, test_play_recursive_proof_gen)
{
    Composer app_composer;
    play_app_circuit(app_composer, 1, 2);

    if (app_composer.failed) {
        info("Play app circuit logic failed: ", app_composer.err);
    }

    UnrolledProver app_prover = app_composer.create_unrolled_prover();
    waffle::plonk_proof app_proof = app_prover.construct_proof();
    info("app_proof: ", app_proof.proof_data);

    std::shared_ptr<waffle::verification_key> app_vk = app_composer.compute_verification_key();

    Composer recursive_composer;
    recursion_output<bn254> recursion_output = play_recursive_circuit(recursive_composer, app_vk, app_proof);

    if (recursive_composer.failed) {
        info("Play recursive circuit logic failed: ", recursive_composer.err);
    }
}

TEST(play_tests, test_play_recursive_2_proof_gen)
{
    Composer app_composer;
    play_app_circuit(app_composer, 1, 2);

    if (app_composer.failed) {
        info("Play app circuit logic failed: ", app_composer.err);
    }

    UnrolledProver app_prover = app_composer.create_unrolled_prover();
    waffle::plonk_proof app_proof = app_prover.construct_proof();
    std::shared_ptr<waffle::verification_key> app_vk = app_composer.compute_verification_key();

    info("Hi 1");

    //*******************************************************************************

    Composer dummy_circuit_composer;
    dummy_circuit(dummy_circuit_composer, 1, 2);

    if (dummy_circuit_composer.failed) {
        info("dummy_circuit logic failed: ", dummy_circuit_composer.err);
    }

    UnrolledProver dummy_circuit_prover = dummy_circuit_composer.create_unrolled_prover();
    waffle::plonk_proof dummy_circuit_proof = dummy_circuit_prover.construct_proof();
    std::shared_ptr<waffle::verification_key> dummy_circuit_vk = dummy_circuit_composer.compute_verification_key();

    info("Hi 2");

    //*******************************************************************************

    Composer recursion_1_composer = Composer("../srs_db/ignition", 0);
    recursion_output<bn254> recursion_1_output =
        play_recursive_circuit_2(recursion_1_composer, app_vk, app_proof, dummy_circuit_vk, dummy_circuit_proof);

    info("Hi 3");
    info("recursion 1 composer gates: ", recursion_1_composer.n);

    if (recursion_1_composer.failed) {
        info("recursion_1 circuit logic failed: ", recursion_1_composer.err);
    }

    UnrolledProver recursion_1_prover = recursion_1_composer.create_unrolled_prover();
    info("Hi 4");

    waffle::plonk_proof recursion_1_proof = recursion_1_prover.construct_proof();
    info("Hi 5");

    std::shared_ptr<waffle::verification_key> recursion_1_vk = recursion_1_composer.compute_verification_key();

    //*******************************************************************************

    // Composer recursion_2_composer;
    // recursion_output<bn254> recursion_2_output = play_recursive_circuit_2<TurboRecursion>(
    //     recursion_2_composer, app_vk, app_proof, recursion_1_vk, recursion_1_proof);

    // if (recursion_2_composer.failed) {
    //     info("recursion_2 circuit logic failed: ", recursion_2_composer.err);
    // }

    // UnrolledProver recursion_2_prover = recursion_2_composer.create_unrolled_prover();
    // waffle::plonk_proof recursion_2_proof = recursion_2_prover.construct_proof();
    // std::shared_ptr<waffle::verification_key> recursion_2_vk = recursion_2_composer.compute_verification_key();
}

} // namespace aztec3::circuits::recursion