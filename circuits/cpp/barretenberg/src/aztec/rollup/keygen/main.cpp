#include "../proofs/escape_hatch/compute_circuit_data.hpp"
#include "../proofs/rollup/compute_circuit_data.hpp"
#include "../proofs/root_rollup/compute_circuit_data.hpp"
#include "../proofs/rollup/rollup_tx.hpp"
#include <common/timer.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/verification_key/sol_gen.hpp>
#include <stdlib/types/turbo.hpp>

using namespace ::rollup::proofs;
using namespace plonk::stdlib::types::turbo;
namespace tx_rollup = ::rollup::proofs::rollup;

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);
    if (args.size() < 4) {
        error("usage: ", args[0], " <inner txs> <max inner num> <output path> [srs path]");
        error("");
        error("Generates solidity contracts containing verification keys for:");
        error("  - The escape hatch.");
        error("  - Rollup circuits containing n inner circuits of size <inner txs>.");
        error(" Where n=1 and doubles until <max inner num>.");
        return 1;
    }
    size_t inner_txs = (size_t)atoi(args[1].c_str());
    size_t max_inner_num = (size_t)atoi(args[2].c_str());
    const std::string output_path = args[3];
    const std::string srs_path = (args.size() >= 5) ? args[4] : "../srs_db/ignition";

    {
        auto escape_hatch_circuit_data = escape_hatch::compute_circuit_data(srs_path);
        auto escape_hatch_class_name = std::string("EscapeHatchVk");
        std::ofstream os(output_path + "/EscapeHatchVk.sol");
        output_vk_sol(os, escape_hatch_circuit_data.verification_key, escape_hatch_class_name);
    }

    auto account_circuit_data = account::compute_circuit_data(srs_path);
    auto join_split_circuit_data = join_split::compute_circuit_data(srs_path);
    auto rollup_circuit_data = tx_rollup::get_circuit_data(
        inner_txs, join_split_circuit_data, account_circuit_data, srs_path, "", true, false, false);

    // Release memory held by proving key, we don't need it.
    rollup_circuit_data.proving_key.reset();

    for (size_t i = 1; i <= max_inner_num; i *= 2) {
        auto root_rollup_circuit_data =
            root_rollup::get_circuit_data(i, rollup_circuit_data, srs_path, "", true, false, false);

        auto rollup_size = i * inner_txs;
        auto class_name = std::string("Rollup") + std::to_string(rollup_size) + "Vk";
        std::ofstream os(output_path + "/" + class_name + ".sol");
        output_vk_sol(os, root_rollup_circuit_data.verification_key, class_name);
    }

    return 0;
}
