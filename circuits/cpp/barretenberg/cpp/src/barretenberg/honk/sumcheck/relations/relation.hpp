#pragma once

namespace proof_system::honk::sumcheck {

// TODO(#226)(Adrian): Remove zeta, alpha as they are not used by the relations.
template <typename FF> struct RelationParameters {
    FF beta;
    FF gamma;
    FF public_input_delta;
};
} // namespace proof_system::honk::sumcheck
