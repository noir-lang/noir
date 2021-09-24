#include "../proofs/rollup/compute_circuit_data.hpp"
#include "../proofs/root_rollup/compute_circuit_data.hpp"
#include "../proofs/root_verifier/compute_circuit_data.hpp"
#include "../proofs/rollup/rollup_tx.hpp"
#include "../proofs/claim/index.hpp"
#include <common/timer.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/verification_key/sol_gen.hpp>

using namespace ::rollup::proofs;
namespace tx_rollup = ::rollup::proofs::rollup;

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);
    if (args.size() < 4) {
        error("usage: ", args[0], " <inner txs> <max inner num> <output path> [srs path]");
        error("");
        error("Generates solidity contracts containing verification keys for:");
        error("  - Rollup circuits containing n inner circuits of size <inner txs>.");
        error(" Where n=1 and doubles until <max inner num>.");
        return 1;
    }
    size_t inner_txs = (size_t)atoi(args[1].c_str());
    size_t max_inner_num = (size_t)atoi(args[2].c_str());
    const std::string output_path = args[3];
    const std::string srs_path = (args.size() >= 5) ? args[4] : "../srs_db/ignition";
    const bool persist = (args.size() >= 6) ? (bool)atoi(args[5].c_str()) : false;

    auto srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(srs_path);
    auto account_cd = account::compute_circuit_data(srs);
    auto join_split_cd = join_split::compute_circuit_data(srs);
    auto claim_cd = claim::get_circuit_data(srs, "./data", true, persist, persist);
    auto rollup_circuit_data = tx_rollup::get_circuit_data(
        inner_txs, join_split_cd, account_cd, claim_cd, srs, "./data", true, persist, persist);

    // Release memory held by proving key, we don't need it.
    rollup_circuit_data.proving_key.reset();

    for (size_t i = 1; i <= max_inner_num; i *= 2) {
        auto root_rollup_circuit_data =
            root_rollup::get_circuit_data(i, rollup_circuit_data, srs, "./data", true, persist, persist);
        auto root_verifier_circuit_data =
            root_verifier::get_circuit_data(root_rollup_circuit_data, srs, "./data", true, persist, persist);
        auto class_name = format("Rollup", inner_txs, "x", i, "VkStandard");
        std::ofstream os(output_path + "/" + class_name + ".sol");
        output_vk_sol_standard(os, root_verifier_circuit_data.verification_key, class_name);
    }

    return 0;
}
