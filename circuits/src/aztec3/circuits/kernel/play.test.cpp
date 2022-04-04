#include "index.hpp"
#include <common/test.hpp>
// #include <numeric/random/engine.hpp>
#include <stdlib/types/turbo.hpp>

namespace aztec3::circuits::kernel {

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
    play_app_circuit(app_composer, 1, 2);

    if (app_composer.failed) {
        info("Play app circuit logic failed: ", app_composer.err);
    }

    UnrolledProver app_prover = app_composer.create_unrolled_prover();
    waffle::plonk_proof app_proof = app_prover.construct_proof();
    info("app_proof: ", app_proof.proof_data);
}

TEST(play_tests, test_play_kernel_proof_gen)
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

    Composer kernel_composer;
    recursion_output<bn254> recursion_output = play_kernel_circuit<TurboRecursion>(kernel_composer, app_vk, app_proof);

    if (kernel_composer.failed) {
        info("Play kernel circuit logic failed: ", kernel_composer.err);
    }
}

// TEST(play_tests, test_play_kernel_2_proof_gen)
// {
//     Composer app_composer;
//     play_app_circuit(app_composer, 1, 2);

//     if (app_composer.failed) {
//         info("Play app circuit logic failed: ", app_composer.err);
//     }

//     UnrolledProver app_prover = app_composer.create_unrolled_prover();
//     waffle::plonk_proof app_proof = app_prover.construct_proof();
//     info("app_proof: ", app_proof.proof_data);

//     std::shared_ptr<waffle::verification_key> app_vk = app_composer.compute_verification_key();

//     Composer mock_kernel_composer = Composer()

//         Composer kernel_composer;
//     recursion_output<bn254> recursion_output = play_kernel_circuit_2<TurboRecursion>(
//         kernel_composer, app_vk, app_proof); // test whether the defaults are acceptable.

//     if (kernel_composer.failed) {
//         info("Play kernel circuit logic failed: ", kernel_composer.err);
//     }
// }

} // namespace aztec3::circuits::kernel