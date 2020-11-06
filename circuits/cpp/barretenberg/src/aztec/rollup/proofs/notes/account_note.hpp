#pragma once
#include <stdlib/types/turbo.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include "note_types.hpp"
#include "note_generator_indices.hpp"

namespace rollup {
namespace proofs {
namespace notes {

using namespace plonk::stdlib::types::turbo;

struct account_note {
    account_note(point_ct const& owner_pub_key, point_ct const& signing_pub_key, bool_ct const& is_real)
        : composer_(*owner_pub_key.x.context)
        , owner_pub_key_(owner_pub_key)
        , signing_pub_key_(signing_pub_key)
        , is_real_(is_real)
        , leaf_data_(&composer_)
        , nullifier_(&composer_)
    {}

    point_ct const& owner_pub_key() const { return owner_pub_key_; }

    point_ct const& signing_pub_key() const { return signing_pub_key_; }

    byte_array_ct const& leaf_data() const
    {
        if (leaf_data_.size()) {
            return leaf_data_;
        }
        return leaf_data_.write(owner_pub_key_.x).write(signing_pub_key_.x);
    }

    field_ct nullifier() const
    {
        if (!nullifier_.get_value().is_zero()) {
            return nullifier_;
        }
        std::vector<field_ct> leaf_elements{
            owner_pub_key_.x,
            signing_pub_key_.x,
        };
        nullifier_ = pedersen::compress(leaf_elements, true, ACCOUNT_NULLIFIER_INDEX);
        return nullifier_;
    }

    void set_public() const
    {
        // Owners public key is shared between notes, so just calling set_public_input only makes it public once.
        // We need it to be exposed as a public input twice. So, create a new public input and assert equality.
        auto owner_x = public_witness_ct(&composer_, owner_pub_key_.x.get_value());
        composer_.assert_equal(owner_pub_key_.x.witness_index, owner_x.witness_index);
        composer_.set_public_input(signing_pub_key_.x.witness_index);
    }

  private:
    Composer& composer_;
    const point_ct owner_pub_key_;
    const point_ct signing_pub_key_;
    bool_ct is_real_;
    mutable byte_array_ct leaf_data_;
    mutable field_ct nullifier_;
};

} // namespace notes
} // namespace proofs
} // namespace rollup