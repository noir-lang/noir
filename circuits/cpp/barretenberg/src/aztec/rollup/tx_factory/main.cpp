#include "../proofs/join_split/index.hpp"
#include "../proofs/rollup/index.hpp"
#include "../proofs/root_rollup/index.hpp"
#include "../proofs/root_verifier/index.hpp"
#include "../world_state/world_state.hpp"
#include "../constants.hpp"
#include "../fixtures/compute_or_load_fixture.hpp"
#include <common/streams.hpp>
#include <iostream>
#include <stdlib/merkle_tree/index.hpp>

using namespace ::rollup::proofs;
using namespace ::rollup::fixtures;
using namespace plonk::stdlib::merkle_tree;
using namespace plonk::stdlib::types::turbo;
namespace tx_rollup = ::rollup::proofs::rollup;
using WorldState = ::rollup::world_state::WorldState<MemoryStore>;

tx_rollup::rollup_tx create_inner_rollup(uint32_t num_txs,
                                         uint32_t rollup_size,
                                         join_split::circuit_data const& join_split_circuit_data,
                                         barretenberg::fr const& data_tree_root,
                                         WorldState& world_state,
                                         bool mock_proofs)
{
    info("Generating a ", rollup_size, " rollup with ", num_txs, " txs...");
    auto proofs = std::vector<std::vector<uint8_t>>(num_txs);
    for (size_t i = 0; i < num_txs; ++i) {
        proofs[i] =
            join_split::create_noop_join_split_proof(join_split_circuit_data, data_tree_root, true, mock_proofs);
    }
    return tx_rollup::create_rollup_tx(world_state, rollup_size, proofs);
}

int main(int argc, char** argv)
{
    using serialize::write;
    WorldState world_state;

    std::vector<std::string> args(argv, argv + argc);

    if (args.size() < 4) {
        info("usage:\n",
             args[0],
             " <num_txs> <inner_rollup_size> <outer_rollup_size> <split_proofs_across_rollups> [output_file]");
        return -1;
    }

    uint32_t num_txs = static_cast<uint32_t>(std::stoul(args[1]));
    const uint32_t inner_rollup_size = static_cast<uint32_t>(std::stoul(args[2]));
    const uint32_t outer_rollup_size = static_cast<uint32_t>(std::stoul(args[3]));
    const bool split_txns_across_rollups = args.size() > 4 ? args[4] == "true" : true;
    const bool mock_proofs = args.size() > 5 ? args[5] == "true" : true;
    const std::string output_file = args[6];

    auto crs = std::make_shared<waffle::DynamicFileReferenceStringFactory>("../srs_db/ignition");
    auto join_split_circuit_data = join_split::get_circuit_data(crs, mock_proofs);
    auto data_root = world_state.data_tree.root();
    world_state.root_tree.update_element(0, data_root);

    Timer timer;

    std::vector<std::vector<uint8_t>> rollups_data;
    const auto num_total_txs = num_txs;
    while (num_txs > 0) {
        auto n = split_txns_across_rollups ? (num_total_txs / outer_rollup_size) : std::min(num_txs, inner_rollup_size);
        num_txs -= n;

        auto rollup =
            create_inner_rollup(n, inner_rollup_size, join_split_circuit_data, data_root, world_state, mock_proofs);

        info("Sending tx rollup request with ", n, " txs...");
        write(std::cout, (uint32_t)0);
        write(std::cout, (uint32_t)inner_rollup_size);
        write(std::cout, rollup);
        info("Sent.");

        std::vector<uint8_t> proof_data;
        bool verified;
        read(std::cin, proof_data);
        read(std::cin, verified);
        if (!verified) {
            throw std::runtime_error("Received an unverified proof.");
        }

        rollups_data.push_back(proof_data);
    }

    auto root_rollup = root_rollup::create_root_rollup_tx(world_state, 0, world_state.defi_tree.root(), rollups_data);

    info("Sending root rollup request...");
    write(std::cout, (uint32_t)1);
    write(std::cout, (uint32_t)inner_rollup_size);
    write(std::cout, (uint32_t)outer_rollup_size);
    write(std::cout, root_rollup);
    info("Sent.");

    std::vector<uint8_t> root_rollup_proof_buf;
    bool verified;
    read(std::cin, root_rollup_proof_buf);
    read(std::cin, verified);
    if (!verified) {
        throw std::runtime_error("Received an unverified root rollup proof.");
    }

    info("Sending root verifier request...");
    write(std::cout, (uint32_t)3);
    write(std::cout, (uint32_t)inner_rollup_size);
    write(std::cout, (uint32_t)outer_rollup_size);
    write(std::cout, root_rollup_proof_buf);
    info("Sent.");

    std::vector<uint8_t> proof_data;
    read(std::cin, proof_data);
    read(std::cin, verified);

    info("Verified: ", verified);
    info("Time taken: ", timer.toString());

    if (!output_file.empty()) {
        std::ofstream of(output_file);
        write(of, proof_data);
        write(of, verified);
        info("Saved proof to: ", output_file);
    }

    return 0;
}
