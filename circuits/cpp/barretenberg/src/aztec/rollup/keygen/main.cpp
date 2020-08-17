#include "../rollup_proofs/compute_join_split_circuit_data.hpp"
#include "../rollup_proofs/compute_rollup_circuit_data.hpp"
#include "../rollup_proofs/rollup_tx.hpp"
#include "../rollup_proofs/verify_rollup.hpp"
#include <common/timer.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/verification_key/sol_gen.hpp>
#include <stdlib/types/turbo.hpp>

using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    size_t rollup_size = (args.size() > 1) ? (size_t)atoi(args[1].c_str()) : 1;
    const std::string srs_path = (args.size() > 2) ? args[2] : "../srs_db/ignition";
    const std::string key_path = (args.size() > 3) ? args[3] : "./data";

    auto inner_circuit_data = compute_or_load_join_split_circuit_data(srs_path, key_path);
    auto circuit_data = compute_or_load_rollup_circuit_data(rollup_size, inner_circuit_data, srs_path, key_path);

    auto class_name = std::string("Rollup") + std::to_string(rollup_size) + "Vk";
    output_vk_sol(std::cout, circuit_data.verification_key, class_name);

    return 0;
}
