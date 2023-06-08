#include "simple.hpp"
#include <barretenberg/plonk/proof_system/types/proof.hpp>
#include <barretenberg/plonk/proof_system/proving_key/serialize.hpp>
#include <barretenberg/common/timer.hpp>
#include <memory>

namespace examples::simple {

using namespace proof_system::plonk;
using namespace stdlib::types;

const size_t CIRCUIT_SIZE = 1 << 19;

void build_circuit(Composer& composer)
{
    while (composer.get_num_gates() <= CIRCUIT_SIZE / 2) {
        plonk::stdlib::pedersen_commitment<Composer>::compress(field_ct(witness_ct(&composer, 1)),
                                                               field_ct(witness_ct(&composer, 1)));
    }
}

Composer* create_composer(std::shared_ptr<barretenberg::srs::factories::CrsFactory> const& crs_factory)
{
    // WARNING: Size hint is essential to perform 512k circuits!
    auto composer = std::make_unique<Composer>(crs_factory, CIRCUIT_SIZE);
    info("building circuit...");
    build_circuit(*composer);

    if (composer->failed()) {
        std::string error = format("composer logic failed: ", composer->err());
        throw_or_abort(error);
    }

    info("public inputs: ", composer->get_public_inputs().size());
    info("composer gates: ", composer->get_num_gates());

    info("computing proving key...");
    auto pk = composer->compute_proving_key();

    return composer.release();
}

proof create_proof(Composer* composer)
{
    Timer timer;
    info("computing proof...");
    auto prover = composer->create_ultra_with_keccak_prover();
    auto proof = prover.construct_proof();
    info("proof construction took ", timer.seconds(), "s");
    return proof;
}

bool verify_proof(Composer* composer, proof_system::plonk::proof const& proof)
{
    info("computing verification key...");
    composer->compute_verification_key();
    auto verifier = composer->create_ultra_with_keccak_verifier();
    auto valid = verifier.verify_proof(proof);
    info("proof validity: ", valid);
    return valid;
}

void delete_composer(Composer* composer)
{
    delete composer;
}

} // namespace examples::simple
