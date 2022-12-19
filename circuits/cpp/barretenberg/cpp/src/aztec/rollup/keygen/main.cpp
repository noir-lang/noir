#include "../proofs/rollup/compute_circuit_data.hpp"
#include "../proofs/root_rollup/compute_circuit_data.hpp"
#include "../proofs/root_verifier/compute_circuit_data.hpp"
#include "../proofs/rollup/rollup_tx.hpp"
#include "../proofs/claim/index.hpp"
#include <common/timer.hpp>
#include <plonk/composer/standard_composer.hpp>
#include <proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/verification_key/sol_gen.hpp>

using namespace ::rollup::proofs;
namespace tx_rollup = ::rollup::proofs::rollup;

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);
    if (args.size() < 4) {
        info(
            "usage: ", args[0], " <num inner txs> <comma separated valid outer sizes> <output path> <mock> [srs path]");
        return 1;
    }
    size_t num_inner_tx = (size_t)atoi(args[1].c_str());
    std::string outer_sizes_raw = args[2];
    // parse list of valid outer sizes
    std::vector<size_t> valid_outer_sizes;
    std::istringstream is(outer_sizes_raw);
    std::string outer_size;
    while (std::getline(is, outer_size, ',')) {
        valid_outer_sizes.emplace_back(std::stoul(outer_size));
    };

    const std::string output_path = args[3];
    const bool mock_proof = (args.size() > 4) ? args[4] == "true" : false;
    const std::string srs_path = (args.size() > 5) ? args[5] : "../srs_db/ignition";

    auto srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(srs_path);

    if (!mock_proof) {
        auto account_cd = account::get_circuit_data(srs);
        auto join_split_cd = join_split::get_circuit_data(srs);
        auto claim_cd = claim::get_circuit_data(srs);
        auto rollup_cd = tx_rollup::get_circuit_data(
            num_inner_tx, join_split_cd, account_cd, claim_cd, srs, "", true, false, false, true, true);

        // Release memory held by proving key, we don't need it.
        rollup_cd.proving_key.reset();

        std::vector<std::shared_ptr<waffle::verification_key>> valid_root_rollup_vks;
        root_rollup::circuit_data root_rollup_cd;
        root_verifier::circuit_data root_verifier_cd;
        for (auto i : valid_outer_sizes) {
            root_rollup_cd.proving_key.reset();
            root_rollup_cd = root_rollup::get_circuit_data(i, rollup_cd, srs, "", true, false, false, true, true);
            valid_root_rollup_vks.emplace_back(root_rollup_cd.verification_key);
        }

        root_verifier_cd = root_verifier::get_circuit_data(
            root_rollup_cd, srs, valid_root_rollup_vks, "", true, false, false, true, true);
        std::replace(outer_size.begin(), outer_size.end(), ',', '_');
        auto class_name = format(mock_proof ? "Mock" : "", "VerificationKey", num_inner_tx, "x", outer_size);
        auto filename = output_path + "/" + class_name + ".sol";
        std::ofstream os(filename);
        output_vk_sol(os, root_verifier_cd.verification_key, class_name);

        info("VK contract written to: ", filename);
    } else {
        // Taking a big shortcut here. Rather than computing all the layers of circuits just to get the number of
        // public inputs, we know the root verifier has 17. This is obviously a bit more brittle but is worth
        // the speed up.
        std::vector<fr> public_inputs(17, fr(1));
        waffle::StandardComposer composer(srs);
        mock::mock_circuit(composer, public_inputs);
        auto vk = composer.compute_verification_key();
        auto class_name = "MockVerificationKey";
        auto filename = output_path + "/" + class_name + ".sol";
        std::ofstream os(filename);
        output_vk_sol(os, vk, class_name);

        info("VK contract written to: ", filename);
    }

    return 0;
}