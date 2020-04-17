#include "../../pedersen_note/pedersen_note.hpp"
#include "../../tx/user_context.hpp"
#include "join_split.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::client_proofs::join_split;

TEST(client_proofs_join_split, test_create)
{
}