#include "../proofs/escape_hatch/compute_escape_hatch_circuit_data.hpp"
#include "../proofs/rollup/compute_rollup_circuit_data.hpp"
#include "../proofs/rollup/rollup_tx.hpp"
#include "../proofs/rollup/verify_rollup.hpp"
#include <common/timer.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/verification_key/sol_gen.hpp>
#include <stdlib/types/turbo.hpp>

using namespace rollup::proofs::join_split;
using namespace rollup::proofs::escape_hatch;
using namespace rollup::proofs::rollup;
using namespace plonk::stdlib::types::turbo;

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);
    if (args.size() <= 1) {
        error("usage: ", args[0], " <rollup size> [srs path]");
        return 1;
    }
    const std::string srs_path = (args.size() > 2) ? args[2] : "../srs_db/ignition";

    if (args[1] == "eh") {
        auto escape_hatch_circuit_data = compute_escape_hatch_circuit_data(srs_path);
        auto escape_hatch_class_name = std::string("EscapeHatchVk");
        output_vk_sol(std::cout, escape_hatch_circuit_data.verification_key, escape_hatch_class_name);
    } else {
        size_t rollup_size = (size_t)atoi(args[1].c_str());

        auto account_circuit_data = compute_account_circuit_data(srs_path);
        auto join_split_circuit_data = compute_join_split_circuit_data(srs_path);
        auto circuit_data =
            compute_rollup_circuit_data(rollup_size, join_split_circuit_data, account_circuit_data, true, srs_path);

        auto class_name = std::string("Rollup") + std::to_string(rollup_size) + "Vk";
        output_vk_sol(std::cout, circuit_data.verification_key, class_name);
    }

    return 0;
}