#include "simple.hpp"
#include <barretenberg/common/timer.hpp>
#include <barretenberg/plonk/proof_system/proving_key/serialize.hpp>
#include <barretenberg/plonk/proof_system/types/proof.hpp>
#include <memory>

namespace examples::simple {

using namespace bb::plonk;
using namespace stdlib::types;

const size_t CIRCUIT_SIZE = 1 << 19;

void build_circuit(Builder& builder)
{
    while (builder.get_num_gates() <= CIRCUIT_SIZE / 2) {
        stdlib::pedersen_hash<Builder>::hash({ field_ct(witness_ct(&builder, 1)), field_ct(witness_ct(&builder, 1)) });
    }
}

BuilderComposerPtrs create_builder_and_composer()
{
    // WARNING: Size hint is essential to perform 512k circuits!
    auto builder = std::make_unique<Builder>(CIRCUIT_SIZE);
    info("building circuit...");
    build_circuit(*builder);

    if (builder->failed()) {
        std::string error = format("builder logic failed: ", builder->err());
        throw_or_abort(error);
    }

    info("public inputs: ", builder->get_public_inputs().size());
    info("composer gates: ", builder->get_num_gates());

    info("computing proving key...");
    auto composer = std::make_unique<Composer>();
    auto pk = composer->compute_proving_key(*builder);

    return { builder.release(), composer.release() };
}

proof create_proof(BuilderComposerPtrs pair)
{
    Timer timer;
    info("computing proof...");
    auto prover = pair.composer->create_ultra_with_keccak_prover(*pair.builder);
    auto proof = prover.construct_proof();
    info("proof construction took ", timer.seconds(), "s");
    return proof;
}

bool verify_proof(BuilderComposerPtrs pair, bb::plonk::proof const& proof)
{
    info("computing verification key...");
    pair.composer->compute_verification_key(*pair.builder);
    auto verifier = pair.composer->create_ultra_with_keccak_verifier(*pair.builder);
    auto valid = verifier.verify_proof(proof);
    info("proof validity: ", valid);
    return valid;
}

void delete_builder_and_composer(BuilderComposerPtrs pair)
{
    delete pair.builder;
    delete pair.composer;
}

} // namespace examples::simple
